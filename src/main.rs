#![no_std]
#![no_main]

use assign_resources::assign_resources;
use embassy_executor::Spawner;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::InterruptHandler;
use embassy_rp::Peri;
use embassy_rp::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

mod adc;
mod deej_usb;
mod graphics;
mod gray4;
mod gray4_effects;
mod screen;
mod sprite;
mod volume_indicator;

assign_resources! {
    screen: ScreenResources {
        spi: SPI1,
        sck: PIN_14,
        mosi: PIN_15,
        miso: PIN_12,
        cs: PIN_13,
        reset: PIN_6,
        pwr: PIN_9,
        dc: PIN_16,
        dma_tx: DMA_CH0,
    },
    adc: AdcResources {
        spi: SPI0,
        sck: PIN_2,
        mosi: PIN_7,
        miso: PIN_4,
        cs: PIN_5,
        dma_tx: DMA_CH1,
        dma_rx: DMA_CH2,
    },
    usb: UsbResources {
        usb: USB
    }
}

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let r = split_resources!(p);

    let (usb_dev, log_class) = deej_usb::init(r.usb);

    spawner.spawn(deej_usb::usb_task(usb_dev).unwrap());
    spawner.spawn(deej_usb::logger_task(log_class).unwrap());

    spawner.spawn(adc::adc_task(r.adc).unwrap());

    screen::init_display_buffers();
    spawner.spawn(screen::render_task(r.screen).unwrap());
    spawner.spawn(graphics::prepare_frame_task().unwrap());
}
