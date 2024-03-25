#![no_std]
#![no_main]

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::mono_font::iso_8859_2::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::{draw_target::DrawTarget, mono_font::MonoTextStyleBuilder};
use esp_backtrace as _;
use hal::{
    clock::ClockControl, peripherals::Peripherals, prelude::*, spi, timer::TimerGroup, Delay, IO,
};
use mipidsi::Orientation;
use mipidsi::{models::ST7789, Display};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;

    let spi = spi::master::Spi::new_no_miso(
        peripherals.SPI2,
        io.pins.gpio13, // TFT_CLK: GPIO13
        io.pins.gpio15, // TFT_MOSI: GPIO15
        io.pins.gpio5,  // TFT_CS: GPIO5
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &clocks,
    );

    let dc = io.pins.gpio14.into_push_pull_output(); // TFT_DC: GPIO14
    let rst = io.pins.gpio12.into_push_pull_output(); // TFT_RST: GPIO12

    let di = SPIInterfaceNoCS::new(spi, dc);
    let mut display: Display<SPIInterfaceNoCS<_, _>, ST7789, _> = mipidsi::Builder::st7789(di)
        .with_display_size(240, 135)
        .with_window_offset_handler(|_| (40, 53))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(rst))
        .unwrap();

    display
        .set_orientation(Orientation::Landscape(true))
        .unwrap();

    display.set_scroll_offset(0).unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::WHITE)
        .build();

    Text::with_baseline("Hello", Point::new(20, 20), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    let mut bl = io.pins.gpio27.into_push_pull_output(); // Brightness
    bl.set_high().unwrap();

    let mut power = io.pins.gpio4.into_push_pull_output(); // Power on
    power.set_high().unwrap();

    timer0.start(1u64.secs());

    loop {
        nb::block!(timer0.wait()).unwrap();
    }
}
