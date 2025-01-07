use adc_mcp3008::{self, Mcp3008};
use embassy_rp::{gpio, peripherals, spi};
use embassy_time::Timer;

use crate::AdcResources;

pub struct AdcMcp<'d> {
    adc: Mcp3008<spi::Spi<'d, peripherals::SPI1, spi::Async>, gpio::Output<'d>>,
}

impl<'d> AdcMcp<'d> {
    pub fn new(adc: AdcResources) -> AdcMcp<'d> {
        let mut cs = gpio::Output::new(adc.cs, gpio::Level::Low);

        let adc_spi = spi::Spi::new(
            adc.spi,
            adc.sck,
            adc.mosi,
            adc.miso,
            adc.dma_tx,
            adc.dma_rx,
            spi::Config::default(),
        );

        cs.set_high();

        Self {
            adc: Mcp3008::new(adc_spi, cs).unwrap(),
        }
    }
}

#[embassy_executor::task]
pub async fn adc_task(adc: AdcResources) {
    let mut adc_mcp = AdcMcp::new(adc);

    loop {
        let r = adc_mcp
            .adc
            .read_channel(adc_mcp3008::Channels8::CH0)
            .unwrap();
        let r1 = r as u32;
        log::info!("{:?}", r1);

        Timer::after_millis(200).await;
    }
}
