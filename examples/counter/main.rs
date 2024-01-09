#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};

use critical_section::Mutex;
use embedded_graphics::{
    geometry::Point,
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_backtrace as _;
use esp_println::println;
use hal::{
    clock::ClockControl,
    gpio::{Event, Gpio15, Input, PullUp},
    i2c::I2C,
    interrupt,
    macros::ram,
    peripherals::{self, Peripherals, I2C0},
    prelude::*,
    timer::TimerGroup,
    IO,
};

use ssd1306::{
    mode::BufferedGraphicsMode, prelude::*, rotation::DisplayRotation, size::DisplaySize128x32,
    I2CDisplayInterface, Ssd1306,
};

type OledDisplay = Ssd1306<
    I2CInterface<I2C<'static, I2C0>>,
    DisplaySize128x32,
    BufferedGraphicsMode<DisplaySize128x32>,
>;

static COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static BUTTON: Mutex<RefCell<Option<Gpio15<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));
static DISPLAY: Mutex<RefCell<Option<OledDisplay>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks = ClockControl::max(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;

    let mut button = io.pins.gpio15.into_pull_up_input();
    button.listen(Event::FallingEdge);

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21,
        io.pins.gpio22,
        100u32.kHz(),
        &clocks,
    );
    let interface = I2CDisplayInterface::new(i2c);
    let mut display: OledDisplay =
        Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Press Button!", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();
    display.clear(BinaryColor::Off).unwrap();

    critical_section::with(|cs| {
        BUTTON.borrow_ref_mut(cs).replace(button);
        DISPLAY.borrow_ref_mut(cs).replace(display);
    });

    interrupt::enable(peripherals::Interrupt::GPIO, interrupt::Priority::Priority2).unwrap();

    // YOU DO NOT NEED CALL IT!
    // https://github.com/esp-rs/esp-hal/discussions/922#discussioncomment-7534573
    // unsafe { xtensa_lx::interrupt::enable(); }

    timer0.start(1u64.secs());

    loop {
        nb::block!(timer0.wait()).unwrap();
    }
}

#[ram]
#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        let count = COUNTER.borrow(cs).get() + 1;
        println!("Button clikced {}", count);
        COUNTER.borrow(cs).set(count);

        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();

        let mut display = DISPLAY.borrow_ref_mut(cs);
        let display = display.as_mut().unwrap();

        display.clear(BinaryColor::Off).unwrap();

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_10X20)
            .text_color(BinaryColor::On)
            .build();

        let mut buffer = itoa::Buffer::new();
        let printed = buffer.format(count);

        Text::with_baseline(printed, Point::zero(), text_style, Baseline::Top)
            .draw(display)
            .unwrap();

        display.flush().unwrap();
    });
}
