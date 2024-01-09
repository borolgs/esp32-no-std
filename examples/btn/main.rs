#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use esp_backtrace as _;
use esp_println::println;
use hal::{
    clock::ClockControl,
    gpio::{Event, Gpio15, Input, PullUp},
    interrupt,
    peripherals::{Interrupt, Peripherals},
    prelude::*,
    Delay, IO,
};

static BUTTON: Mutex<RefCell<Option<Gpio15<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut button = io.pins.gpio15.into_pull_up_input();

    button.listen(Event::FallingEdge);
    interrupt::enable(Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();

    critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));

    let mut delay = Delay::new(&clocks);

    loop {
        println!("Loop");
        delay.delay_ms(2000_u32);
    }

    #[interrupt]
    fn GPIO() {
        println!("Button Press Interrupt");
        // Start a Critical Section
        critical_section::with(|cs| {
            BUTTON
                .borrow_ref_mut(cs)
                .as_mut()
                .unwrap()
                .clear_interrupt();
        });
    }
}
