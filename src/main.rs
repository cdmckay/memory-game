#![no_std]
#![no_main]

// Pull in the panic handler from panic-halt
extern crate panic_halt;

use arduino_uno::{adc, hal::port::Pin};
use arduino_uno::{hal::port::mode::Output, prelude::*};
use oorandom;

// The number of levels
const MAX_LEVELS: usize = 10;

// The amount of time the LED will show
const LED_DURATION_MS: usize = 500;

#[arduino_uno::entry]
fn main() -> ! {
    let dp = arduino_uno::Peripherals::take().unwrap();
    let mut pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);

    let button1 = pins.d2.into_pull_up_input(&mut pins.ddr);
    let button2 = pins.d3.into_pull_up_input(&mut pins.ddr);
    let button3 = pins.d4.into_pull_up_input(&mut pins.ddr);
    let button4 = pins.d5.into_pull_up_input(&mut pins.ddr);
    let buttons = [
        button1.downgrade(),
        button2.downgrade(),
        button3.downgrade(),
        button4.downgrade(),
    ];
    let mut button_states: [bool; 4] = [false; 4];

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
    let mut rng = oorandom::Rand32::new(nb::block!(adc.read(&mut a0)).void_unwrap());

    let mut game_running = false;
    let mut game_waiting = false;

    let mut current_level = 1;
    let mut speed_factor = 5;

    let mut correct = false;

    let mut signals: [u32; MAX_LEVELS] = [0; MAX_LEVELS];
    let mut user_signals: [u32; MAX_LEVELS] = [0; MAX_LEVELS];

    loop {
        // This generates the LED signal pattern randomly
        if !game_running {
            for i in 0..MAX_LEVELS {
                signals[i] = rng.rand_range(0..4);
            }
            game_running = true;
        }
        // This shows the LED signal pattern
        if !game_waiting {
            show_led_signal_pattern(&signals, &mut leds, current_level, speed_factor);
            game_waiting = true;
        }
        // Check for button presses
        let mut button_pressed = false;
        let mut signal_index = 0;
        while signal_index < current_level {
            while !button_pressed {
                for i in 0..4 {
                    button_states[i] = buttons[i].is_high().void_unwrap();
                    button_pressed = button_pressed || button_states[i];
                }
            }
            for i in 0..4 {
                if button_states[i] {
                    let pin = &mut leds[i];
                    pin.set_high().void_unwrap();
                    arduino_uno::delay_ms(LED_DURATION_MS as u16);
                    pin.set_low().void_unwrap();
                    game_waiting = false;
                    user_signals[signal_index] = i as u32;
                    button_pressed = false;
                }
            }
            if user_signals[signal_index] == signals[signal_index] {
                signal_index += 1;
                correct = true;
            } else {
                correct = false;
                signal_index = current_level;
                game_waiting = false;
            }
        }
        if correct {
            current_level += 1;
            game_waiting = false;
        } else {
            current_level = 1;
            game_running = false;
            arduino_uno::delay_ms(300);
            for i in 0..4 {
                leds[i].set_high().void_unwrap();
            }
            arduino_uno::delay_ms(LED_DURATION_MS as u16);
            for i in 0..4 {
                leds[i].set_low().void_unwrap();
            }
            arduino_uno::delay_ms(200);
            for i in 0..4 {
                leds[i].set_high().void_unwrap();
            }
            arduino_uno::delay_ms(LED_DURATION_MS as u16);
            for i in 0..4 {
                leds[i].set_low().void_unwrap();
            }
            arduino_uno::delay_ms(500);
        }

        if current_level == MAX_LEVELS {
            game_running = false;
            current_level = 1;
            speed_factor += 1;
            arduino_uno::delay_ms(300);
            for i in 0..4 {
                leds[i].set_high().void_unwrap();
                arduino_uno::delay_ms(200);
            }
            for i in 0..4 {
                leds[i].set_low().void_unwrap();
                arduino_uno::delay_ms(200);
            }
            arduino_uno::delay_ms(500);
        }
    }
}

fn show_led_signal_pattern(
    signals: &[u32],
    leds: &mut [Pin<Output>],
    current_level: usize,
    speed_factor: usize,
) -> () {
    arduino_uno::delay_ms(200);
    let led_delay_ms = LED_DURATION_MS / (1 + (speed_factor / MAX_LEVELS) * (current_level - 1));
    for i in 0..current_level {
        let index = signals[i] as usize;
        let pin = &mut leds[index];
        pin.set_high().void_unwrap();
        arduino_uno::delay_ms(led_delay_ms as u16);
        pin.set_low().void_unwrap();
        arduino_uno::delay_ms(100 / speed_factor as u16);
    }
}
