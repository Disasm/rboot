//! This bootloader code based on `double_tap_dontboot` from freedom-e-sdk
//! https://github.com/sifive/freedom-e-sdk/blob/v20180402/software/double_tap_dontboot/double_tap_dontboot.c
//!
//! These are the instructions for the user of this program, from the
//! HiFive1 Getting Started Guide:
//!
//! This program is designed to allow quick boot, but
//! also a "safe" reboot option if a "bad" program
//! is flashed into the HiFive1's SPI Flash. A "bad" program
//! is one which makes it impossible for the programmer
//! to communicate with the HiFive1. For example, a program which
//! disables FE310's active clock, or which puts the FE310 to sleep
//! with no way of waking it up. Bad programs can always be restarted using
//! the RESET button, and using the "safe" bootloader can be halted
//! before they perform any unsafe behavior.
//!
//! To activate "normal" boot mode, press the RESET button on
//! the HiFive1. After approximately 1s, the green LED will flash
//! for 1/2 second, then the user program will execute.
//!
//! To activate "safe" boot mode, press the RESET button. When
//! the green LED flashes, immediately press the RESET button again.
//! After 1 second, the red LED will blink. The user program will not
//! execute, and the programmer can connect. To exit "safe" boot mode,
//! press the RESET button a final time.

#![no_std]
#![no_main]

extern crate panic_halt;

use riscv_rt::entry;
use e310x_hal::e310x::Peripherals;
use e310x_hal::delay::Delay;
use riscv::register::mtvec;
use embedded_hal::blocking::delay::DelayMs;
use bit_field::BitField;

const BACKUP15_MAGIC: u32 = 0xD027B007;

#[cfg(feature = "board-hifive1")]
const FINAL_ADDRESS: usize = 0x20400000;
#[cfg(feature = "board-hifive1-revb")]
const FINAL_ADDRESS: usize = 0x20010000;

const RED_LED: usize = 22;
const GREEN_LED: usize = 19;

#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();

    // Restore the default mtvec (which may have been set by initialization code.
    // By default, this would cause an infinite loop upon exception, which is also "safe" behavior
    // and the debugger can connect.
    unsafe { mtvec::write(0, mtvec::TrapMode::Direct); }

    // How did we get here? We only want to execute this code
    // on resets (vs waking up from sleep).
    if p.PMU.pmucause.read().wakeupcause().is_reset() {
        if p.BACKUP.backup[15].read().bits() == BACKUP15_MAGIC {
            // Reset was "double-tapped".

            p.BACKUP.backup[15].write(|w| unsafe { w.bits(0) });

            // PWM Red LED
            #[cfg(any(feature = "board-hifive1", feature = "board-hifive1-revb"))]
            unsafe {
                p.GPIO0.iof_en.modify(|r, w| w.bits(*r.bits().set_bit(RED_LED, true)));
                p.GPIO0.out_xor.modify(|r, w| w.bits(*r.bits().set_bit(RED_LED, false)));
                p.GPIO0.iof_sel.modify(|r, w| w.bits(*r.bits().set_bit(RED_LED, true)));

                p.GPIO0.port.modify(|r, w| w.bits(*r.bits().set_bit(GREEN_LED, false)));
                p.GPIO0.out_xor.modify(|r, w| w.bits(*r.bits().set_bit(GREEN_LED, false)));
                p.GPIO0.output_en.modify(|r, w| w.bits(*r.bits().set_bit(GREEN_LED, false)));

                p.PWM1.cfg.write(|w| w.bits(0));
                p.PWM1.count.write(|w| w.bits(0));
                p.PWM1.cmp0.write(|w| w.bits(0xff));
                p.PWM1.cmp3.write(|w| w.bits(0xff));
                p.PWM1.cfg.write(|w| w.enalways().set_bit());
            }

            let mut pwm_val: u8 = 255;

            // Wait for debugger or another RESET press.
            loop {
                // Make the PWM a fade. This is preferable to just a PWM blink
                // because it makes it clear that the processor is actually
                // running this code, not just the PWM hardware.

                Delay.delay_ms(2u8);
                pwm_val = pwm_val.wrapping_sub(1);
                unsafe {
                    p.PWM1.cmp3.write(|w| w.value().bits((pwm_val as u16) << 3));
                }
            }
        }

        // Save previous backup register value
        let save = p.BACKUP.backup[15].read().bits();

        p.BACKUP.backup[15].write(|w| unsafe { w.bits(BACKUP15_MAGIC) });

        // Wait 500 ms. If reset is tapped at this point,
        // we will execute the "magic" loop above.
        Delay.delay_ms(500u32);

        // Restore previous backup register value
        p.BACKUP.backup[15].write(|w| unsafe { w.bits(save) });
    }

    // Restore the GPIO Registers to their default
    #[cfg(any(feature = "board-hifive1", feature = "board-hifive1-revb"))]
    unsafe {
        p.GPIO0.port.write(|w| w.bits(0));
        p.GPIO0.out_xor.write(|w| w.bits(0));
        p.GPIO0.output_en.write(|w| w.bits(0));
    }

    // Jump to "user code" in SPI Flash.
    let pgm_start = FINAL_ADDRESS as *const fn() -> !;
    unsafe { (*pgm_start)() }
}
