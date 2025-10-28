mod keypad;
mod layout;

use atomic_enum::atomic_enum;
use std::ffi::{c_char, c_int};

use keypad::Keypad as KeypadDriver;

#[repr(C)]
#[atomic_enum]
pub enum Lock {
    Locked = 0,
    Unlocked = 1,
    UnlockedPowerOnly = 2,
}

pub struct Keypad {
    driver: Option<KeypadDriver>,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keypad_new() -> *mut Keypad {
    let keypad = Keypad { driver: None };
    Box::into_raw(Box::new(keypad))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keypad_delete(kp: *mut Keypad) {
    unsafe {
        let kp = Box::from_raw(kp);
        if let Some(drv) = kp.driver {
            drv.stop();
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keypad_init(kp: *mut Keypad) -> c_int {
    let kp = unsafe { &mut *kp };
    match KeypadDriver::open() {
        Ok(drv) => {
            kp.driver = Some(drv);
            1
        }

        Err(e) => {
            eprintln!("Keypad open error: {e}");
            0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keypad_run(kp: *mut Keypad) {
    let kp = unsafe { &mut *kp };
    if let Some(ref mut drv) = kp.driver {
        if let Err(e) = drv.scan() {
            eprintln!("Keypad scan error: {e}");
        }
    } else {
        eprintln!("Keypad not initialized");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keypad_set_lock(kp: *mut Keypad, lock: Lock) {
    let kp = unsafe { &mut *kp };
    if let Some(ref mut drv) = kp.driver {
        drv.set_lock(lock)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keypad_get_lock(kp: *mut Keypad) -> Lock {
    let kp = unsafe { &mut *kp };
    if let Some(ref mut drv) = kp.driver {
        drv.get_lock()
    } else {
        Lock::Locked
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keypad_set_on_pressed(kp: *mut Keypad) {
    let kp = unsafe { &mut *kp };
    if let Some(ref mut drv) = kp.driver {
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keypad_set_on_released(kp: *mut Keypad) {
    let kp = unsafe { &mut *kp };
    if let Some(ref mut drv) = kp.driver {
    }
}
