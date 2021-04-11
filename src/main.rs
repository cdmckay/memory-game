#![no_std]
#![no_main]

// Pull in the panic handler from panic-halt
extern crate panic_halt;

use arduino_uno::adc;
use arduino_uno::prelude::*;
use rand_core::{RngCore, SeedableRng};
use wyhash::WyRng;

// The number of levels
const MAX_LEVELS: usize = 10;

#[arduino_uno::entry]
fn main() -> ! {
    let dp = arduino_uno::Peripherals::take().unwrap();
    let mut pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);

    let led1 = pins.d7.into_output(&mut pins.ddr);
    let led2 = pins.d8.into_output(&mut pins.ddr);
    let led3 = pins.d9.into_output(&mut pins.ddr);
    let led4 = pins.d10.into_output(&mut pins.ddr);
    let mut leds = [
        led1.downgrade(),
        led2.downgrade(),
        led3.downgrade(),
        led4.downgrade(),
    ];

    let mut adc = adc::Adc::new(dp.ADC, Default::default());
    let mut a0 = pins.a0.into_analog_input(&mut adc);
    let mut rng = WyRng::seed_from_u64(nb::block!(adc.read(&mut a0)).void_unwrap());

    let mut game_running = false;
    let mut game_waiting = false;

    let mut current_level = 4;

    loop {
        let mut signals: [u32; MAX_LEVELS] = [0; MAX_LEVELS];
        //let mut user_signals: [u32; MAX_LEVELS] = [0; MAX_LEVELS];

        if !game_running {
            for i in 0..MAX_LEVELS {
                signals[i] = rng.next_u32() % 4;
            }
            game_running = true;
        }
        if !game_waiting {
            arduino_uno::delay_ms(200);
            for i in 0..current_level {
                let index = signals[i] as usize;
                let pin = &mut leds[index];
                pin.set_high().void_unwrap();
                arduino_uno::delay_ms(1000);
                pin.set_low().void_unwrap();
            }
            game_waiting = true;
        }
    }
}
