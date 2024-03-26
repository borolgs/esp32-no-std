#![no_std]
#![no_main]

mod display;

use core::cell::{Cell, RefCell};

use critical_section::Mutex;
use display::{display, StickDisplay};
use embedded_graphics::mono_font::iso_8859_2::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::{draw_target::DrawTarget, mono_font::MonoTextStyleBuilder};
use esp_backtrace as _;

use esp_println::println;
use hal::gpio::{Event, Gpio37, Input, PullUp};
use hal::interrupt;
use hal::peripherals::Interrupt;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, timer::TimerGroup, IO};

static COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static BUTTON_A: Mutex<RefCell<Option<Gpio37<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));
static DISPLAY: Mutex<RefCell<Option<StickDisplay>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks = ClockControl::max(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;

    let mut button_a = io.pins.gpio37.into_pull_up_input();
    button_a.listen(Event::FallingEdge);

    let mut display = display(
        clocks,
        peripherals.SPI2,
        io.pins.gpio13,
        io.pins.gpio15,
        io.pins.gpio5,
        io.pins.gpio14,
        io.pins.gpio12,
    );

    display.clear(Rgb565::BLACK).unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::WHITE)
        .build();

    Text::with_baseline(
        "Press Button!",
        Point::new(20, 20),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    let mut bl = io.pins.gpio27.into_push_pull_output(); // Brightness
    bl.set_high().unwrap();

    let mut hold = io.pins.gpio4.into_push_pull_output(); // Power on
    hold.set_high().unwrap();

    critical_section::with(|cs| {
        BUTTON_A.borrow_ref_mut(cs).replace(button_a);
        DISPLAY.borrow_ref_mut(cs).replace(display);
    });
    interrupt::enable(Interrupt::GPIO, interrupt::Priority::Priority2).unwrap();

    timer0.start(1u64.secs());

    loop {
        nb::block!(timer0.wait()).unwrap();
    }
}

#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        let count = COUNTER.borrow(cs).get() + 1;
        println!("Button clikced {}", count);
        COUNTER.borrow(cs).set(count);

        BUTTON_A
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();

        let mut display = DISPLAY.borrow_ref_mut(cs);
        let display = display.as_mut().unwrap();

        display.clear(Rgb565::BLACK).unwrap();

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_10X20)
            .text_color(Rgb565::WHITE)
            .build();

        let mut buffer = itoa::Buffer::new();
        let printed = buffer.format(count);

        Text::with_baseline(printed, Point::new(20, 20), text_style, Baseline::Top)
            .draw(display)
            .unwrap();
    });
}
