#![no_std]
#![no_main]

use assign_resources::assign_resources;
use embassy_executor::Spawner;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

mod adc;
mod animation_node;
mod graphics;
mod screen;

assign_resources! {
    screen: ScreenResources {
        spi: SPI0,
        sck: PIN_2,
        mosi: PIN_3,
        miso: PIN_16,
        cs: PIN_4,
        reset: PIN_6,
        pwr: PIN_9,
        dc: PIN_5,
        dma_tx: DMA_CH0,
    },
    adc: AdcResources {
        spi: SPI1,
        sck: PIN_14,
        mosi: PIN_15,
        miso: PIN_12,
        cs: PIN_13,
        dma_tx: DMA_CH1,
        dma_rx: DMA_CH2,
    }
}

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let r = split_resources!(p);

    let driver = Driver::new(p.USB, Irqs);
    spawner.must_spawn(logger_task(driver));

    spawner.must_spawn(adc::adc_task(r.adc));

    screen::init_display_buffers();
    spawner.must_spawn(screen::render_task(r.screen));
    spawner.must_spawn(graphics::prepare_frame_task());
}
