#pragma once

#ifndef KEYPAD_H
#define KEYPAD_H

#include "ckeypad"

typedef void (*key_event_handler) (char, uint32_t);

class keypad {
public:
  // Represents the locking mode
  enum locking_mode {
      UNLOCKED  = (int)Lock::Unlocked,           // All keys unlocked
      LOCKED    = (int)Lock::Locked,             // All keys locked
      ON_OFF    = (int)Lock::UnlockedPowerOnly   // All keys locked except for the power key
  };

  static int          Initialize                (int)                        // Initializes the keypad. Legacy int argument is unused.
  {
      kp = ::keypad_new();
      return !::keypad_init(kp);
  }

  static void         Run                       ()                           // Runs the main loop which addresses the mux pins and listens on the read pins.
  {
      ::keypad_run(kp);
  }

  static locking_mode GetLock                   ()                           // Returns the current locking status.
  {
      return (locking_mode)::keypad_get_lock(kp);
  }

  static void         SetLock                   (locking_mode mode)          // Locks or unlocks the keypad.
  {
      ::keypad_set_lock(kp, static_cast<Lock>(mode));
  }

  static void         Terminate                 ()                           // Terminates the keypad driver.
  {
      ::keypad_delete(kp);
  }

  static void         SetKeyPressEventHandler   (key_event_handler handler)  // Sets the handler for key press events.
  {
      ::keypad_set_on_pressed(kp, handler, 0);
  }

  static void         SetKeyReleaseEventHandler (key_event_handler handler)  // Sets the handler for key release events.
  {
      ::keypad_set_on_released(kp, handler, 0);
  }
private:
  static struct Keypad *kp;
};

#endif
