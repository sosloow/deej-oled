use adc_mcp3008::{self, Channels8, Mcp3008};
use core::sync::atomic::{AtomicU32, Ordering};
use embassy_rp::{gpio, peripherals, spi};
use embassy_time::Timer;

use crate::AdcResources;

// Static shared state accessible from anywhere
pub static ADC_VALUES: [AtomicU32; 8] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

pub static ADC_CHANNELS: [Channels8; 1] = [Channels8::CH0];

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
        // Read all channels in a loop
        for (i, channel) in ADC_CHANNELS.iter().enumerate() {
            if let Ok(value) = adc_mcp.adc.read_channel(*channel) {
                ADC_VALUES[i].store(value as u32, Ordering::Relaxed);
            } else {
                log::warn!("Failed to read channel {}", i);
            }
        }

        log::info!("{}", ADC_VALUES[0].load(Ordering::Relaxed));

        Timer::after_millis(200).await;
    }
}

pub fn read_adc_value(channel: usize) -> u32 {
    if channel < 8 {
        ADC_VALUES[channel].load(Ordering::Relaxed)
    } else {
        0
    }
}
