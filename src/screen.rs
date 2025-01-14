use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use ssd1322_rs::{self, calculate_buffer_size, Frame, Orientation, SSD1322};
use static_cell::StaticCell;

use super::ScreenResources;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 64;
const BUF_SIZE: usize = calculate_buffer_size(SCREEN_WIDTH, SCREEN_HEIGHT);

static FRAME_A: StaticCell<Frame<BUF_SIZE>> = StaticCell::new();
static FRAME_B: StaticCell<Frame<BUF_SIZE>> = StaticCell::new();

pub static NEXT_FRAME: Signal<ThreadModeRawMutex, &'static mut Frame<BUF_SIZE>> = Signal::new();
pub static READY_FRAME: Signal<ThreadModeRawMutex, &'static mut Frame<BUF_SIZE>> = Signal::new();

pub fn init_display_buffers() {
    let frame_a = FRAME_A.init(Default::default());
    NEXT_FRAME.signal(frame_a);

    let frame_b = FRAME_B.init(Default::default());
    READY_FRAME.signal(frame_b);
}

#[embassy_executor::task]
pub async fn render_task(screen: ScreenResources) {
    let mut spi_config = spi::Config::default();
    spi_config.frequency = 10_000_000;

    let spi_p = spi::Spi::new_txonly(
        screen.spi,
        screen.sck,
        screen.mosi,
        screen.dma_tx,
        spi_config,
    );
    let reset = Output::new(screen.reset, Level::Low);
    let scr_power = Output::new(screen.pwr, Level::Low);
    let data_command_pin = Output::new(screen.dc, Level::Low);
    let cs_pin = Output::new(screen.cs, Level::Low);

    let spi_dev = ExclusiveDevice::new_no_delay(spi_p, cs_pin);

    let mut display = SSD1322::new(
        spi_dev,
        data_command_pin,
        reset,
        scr_power,
        Default::default(),
    );
    display.init_default(&mut Delay).await.unwrap();
    display
        .set_orientation(Orientation::Inverted)
        .await
        .unwrap();

    let mut frame = READY_FRAME.wait().await;

    loop {
        NEXT_FRAME.signal(frame);
        frame = READY_FRAME.wait().await;

        match display.flush_frame(frame).await {
            Ok(_) => (),
            Err(_e) => (),
        }
    }
}
