use display_interface_spi::SPIInterfaceNoCS;

use esp_backtrace as _;
use hal::clock::Clocks;
use hal::gpio::{GpioPin, Output, PushPull, Unknown};
use hal::peripherals::SPI2;
use hal::spi::master::Spi;
use hal::spi::FullDuplexMode;
use hal::{prelude::*, spi, Delay};
use mipidsi::Orientation;
use mipidsi::{models::ST7789, Display};

pub type StickDisplay<'a> = Display<
    SPIInterfaceNoCS<Spi<'a, SPI2, FullDuplexMode>, GpioPin<Output<PushPull>, 14>>,
    ST7789,
    GpioPin<Output<PushPull>, 12>,
>;

/// Initialize the display
pub fn display(
    clocks: Clocks,
    spi: SPI2,
    // TFT_CLK: GPIO13 - SPI clock
    clk: GpioPin<Unknown, 13>,
    // TFT_MOSI: GPIO15 - Master(Controller) Output Slave(Display) Input
    mosi: GpioPin<Unknown, 15>,
    // TFT_CS: GPIO5 - Chip select is a control signal used to select a specific device on the SPI bus
    cs: GpioPin<Unknown, 5>,
    // TFT_DC: GPIO14 - Data/Command
    dc: GpioPin<Unknown, 14>,
    // TFT_RST: GPIO12 - Reset
    rst: GpioPin<Unknown, 12>,
) -> StickDisplay {
    let mut delay = Delay::new(&clocks);

    // Serial Peripheral Interface (SPI)
    // MISO - Master Input Slave Output bus
    let spi = spi::master::Spi::new_no_miso(
        spi,
        clk,
        mosi,
        cs,
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &clocks,
    );

    let dc = dc.into_push_pull_output();
    let rst = rst.into_push_pull_output();

    let di = SPIInterfaceNoCS::new(spi, dc);
    let mut display: StickDisplay = mipidsi::Builder::st7789(di)
        .with_display_size(240, 135)
        .with_window_offset_handler(|_| (40, 53))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(rst))
        .unwrap();

    display
        .set_orientation(Orientation::Landscape(true))
        .unwrap();

    display.set_scroll_offset(0).unwrap();

    display
}
