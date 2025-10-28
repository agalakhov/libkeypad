use anyhow::Error;
use i2cdev::{
    core::{I2CDevice, I2CMessage, I2CTransfer},
    linux::LinuxI2CDevice,
};
use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use super::{
    AtomicLock, Lock,
    layout::{Symbol, translate},
};

const DEVICE: &str = "/dev/i2c-1";
const ADDRESS: u16 = 0b_010_0_000;

const ROWS: [(usize, usize); 8] = [
    (1, 1),
    (1, 4),
    (1, 3),
    (2, 2),
    (2, 1),
    (2, 4),
    (2, 3),
    (1, 2),
];
const COLS: [usize; 3] = [2, 1, 3];

/// Register of the chip (selection).
#[allow(dead_code)]
enum Reg {
    DirA = 0x00,
    DirB = 0x10,
    OutA = 0x0A,
    OutB = 0x1A,
    PupA = 0x06,
    PupB = 0x16,
    InpA = 0x09,
    InpB = 0x19,
}

pub struct Keypad {
    dev: Mutex<LinuxI2CDevice>,
    lock_state: AtomicLock,
    on_pressed: Mutex<Option<Box<dyn FnMut(Symbol) + Send>>>,
    on_released: Mutex<Option<Box<dyn FnMut(Symbol) + Send>>>,
    stop: AtomicBool,
}

impl Keypad {
    /// Open and initialize keypad.
    pub fn open() -> Result<Self, Error> {
        let mut dev = LinuxI2CDevice::new(DEVICE, ADDRESS)?;

        // Set MCP23017 to predictable state.
        dev.write(&[0x05, 0b1000_0000])?; // IOCON BANK=1
        dev.write(&[0x0A, 0b1000_0000])?; // IOCON BANK=1
        dev.write(&[0x0A, 0b0000_0000])?; // OLATA
        dev.write(&[0x12, 0b0000_0000])?; // INTCONB (aka 0x05)

        let dev = Mutex::new(dev);
        Ok(Self {
            dev,
            stop: AtomicBool::new(false),
            lock_state: AtomicLock::new(Lock::Unlocked),
            on_pressed: Mutex::new(None),
            on_released: Mutex::new(None),
        })
    }

    /// Run scanning thread.
    pub fn scan(&self) -> Result<(), Error> {
        let mut matrix: [[[bool; 3]; 4]; 2] = Default::default();
        let mut dev = self.dev.lock().unwrap();
        self.stop.store(false, Ordering::SeqCst);

        // Pre-charge capacitors to avoid false positives.
        dev.write_reg(Reg::DirB, 0xFF)?; // port B as input (hi-Z)
        dev.write_reg(Reg::DirA, 0x00)?; // port A temporarily as output
        dev.write_reg(Reg::OutA, 0xFF)?; // port A all lines HIGH
        sleep(Duration::from_millis(10));

        // Reconfigure port A as input
        dev.write_reg(Reg::DirA, 0xFF)?; // port A as input
        dev.write_reg(Reg::PupA, 0xFF)?; // port A all pull-ups on

        while !self.stop.load(Ordering::SeqCst) {
            for (scanrow, (pad, row)) in ROWS.iter().enumerate() {
                let m = !(1u8 << scanrow);
                dev.write_reg(Reg::DirB, m)?;
                dev.write_reg(Reg::OutB, m)?;
                sleep(Duration::from_millis(5));

                let byte = {
                    let mut buf = [0];
                    let mut ops = [
                        I2CMessage::write(&[Reg::InpA as u8]),
                        I2CMessage::read(&mut buf),
                    ];
                    dev.transfer(&mut ops)?;
                    buf[0]
                };

                let input = [
                    byte & (1 << 1) == 0,
                    byte & (1 << 4) == 0,
                    byte & (1 << 7) == 0,
                ];
                let columns: &mut [bool; 3] = &mut matrix[*pad][*row];
                for (i, &pressed) in input.iter().enumerate() {
                    let idx = COLS[i];
                    match (columns[idx], pressed) {
                        (false, true) => {
                            let chr = translate(*pad, *row, idx);
                            if !self.is_locked(chr) {
                                columns[idx] = pressed;
                                if let Some(ref mut cb) = *self.on_pressed.lock().unwrap() {
                                    cb(chr)
                                }
                            }
                        }
                        (true, false) => {
                            columns[idx] = pressed;
                            let chr = translate(*pad, *row, idx);
                            if let Some(ref mut cb) = *self.on_released.lock().unwrap() {
                                cb(chr)
                            }
                        }
                        _ => {}
                    }
                }
                // Re-charge capacitors
                dev.write_reg(Reg::OutB, 0xFF)?;
                dev.write_reg(Reg::OutA, 0xFF)?;
                dev.write_reg(Reg::DirA, 0x00)?;
                sleep(Duration::from_millis(1));
                dev.write_reg(Reg::DirA, 0xFF)?;
            }
        }
        Ok(())
    }

    /// Set lock status.
    pub fn set_lock(&self, lock: Lock) {
        self.lock_state.store(lock, Ordering::Relaxed)
    }

    /// Get lock status.
    pub fn get_lock(&self) -> Lock {
        self.lock_state.load(Ordering::Relaxed)
    }

    /// Stop polling thread.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        // Wait until the device is released. Try to lock the mutex.
        let lock = self.dev.lock().unwrap();
        // Immediatelu release the mutex.
        drop(lock);
    }

    /// Set `OnPressed` callback.
    pub fn set_on_pressed(&self, cb: Box<dyn FnMut(Symbol) + Send>) {
        *self.on_pressed.lock().unwrap() = Some(cb)
    }

    /// Set `OnReleased` callback.
    pub fn set_on_released(&self, cb: Box<dyn FnMut(Symbol) + Send>) {
        *self.on_released.lock().unwrap() = Some(cb)
    }

    /// Check if the keyboard is locked for the given character.
    fn is_locked(&self, chr: Symbol) -> bool {
        let lock = self.lock_state.load(Ordering::Relaxed);
        match lock {
            Lock::Unlocked => false,
            Lock::UnlockedPowerOnly if chr.is_power() => false,
            _ => true,
        }
    }
}

/// Convenience trait for register-level access.
trait RegAccess: I2CDevice {
    fn write_reg(&mut self, reg: Reg, val: u8) -> Result<(), <Self as I2CDevice>::Error>;
}

impl<D> RegAccess for D
where
    D: I2CDevice,
{
    fn write_reg(&mut self, reg: Reg, val: u8) -> Result<(), <Self as I2CDevice>::Error> {
        self.write(&[reg as u8, val])
    }
}
