use adc_mcp3008::{self, Channels8, Mcp3008};
use core::cmp::{max, min};
use core::fmt::Write as _;
use core::sync::atomic::{AtomicU32, Ordering};
use embassy_rp::{gpio, peripherals, spi};
use embassy_time::Timer;
use heapless::String;

use crate::AdcResources;

#[derive(Clone, Copy)]
pub struct AdcChanCfg {
    pub invert: bool,
    pub min: u16,
    pub max: u16,
    pub chan: Channels8,
}

pub const NOISE_THRESHOLD: u32 = 15;

pub const ADC_CHANNELS: [AdcChanCfg; 5] = [
    AdcChanCfg {
        invert: true,
        min: 0,
        max: 850,
        chan: Channels8::CH0,
    },
    AdcChanCfg {
        invert: true,
        min: 0,
        max: 1023,
        chan: Channels8::CH1,
    },
    AdcChanCfg {
        invert: true,
        min: 0,
        max: 1023,
        chan: Channels8::CH2,
    },
    AdcChanCfg {
        invert: true,
        min: 0,
        max: 1023,
        chan: Channels8::CH3,
    },
    AdcChanCfg {
        invert: true,
        min: 0,
        max: 1023,
        chan: Channels8::CH4,
    },
];

// Static shared state accessible from anywhere
pub static ADC_VALUES: [AtomicU32; 5] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

pub struct AdcMcp<'d> {
    adc: Mcp3008<spi::Spi<'d, peripherals::SPI0, spi::Async>, gpio::Output<'d>>,
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
        let mut snapshot: [u32; ADC_VALUES.len()] = [0; ADC_VALUES.len()];
        for (i, s) in snapshot.iter_mut().enumerate() {
            *s = ADC_VALUES[i].load(Ordering::Relaxed);
        }

        let mut any_updated = false;

        for (i, conf) in ADC_CHANNELS.iter().enumerate() {
            if let Ok(raw) = adc_mcp.adc.read_channel(conf.chan) {
                let norm = normalize_value(raw, *conf);
                let curr = snapshot[i];
                let diff = max(curr, norm) - min(curr, norm);

                if diff >= NOISE_THRESHOLD {
                    ADC_VALUES[i].store(norm, Ordering::Relaxed);
                    snapshot[i] = norm;
                    any_updated = true;
                }
            } else {
                log::warn!("Failed to read channel {}", i);
            }
        }

        if any_updated {
            let mut out: String<96> = String::new();
            for (i, v) in snapshot.iter().enumerate() {
                if i > 0 {
                    let _ = out.push('|');
                }
                let _ = write!(out, "{}", v);
            }
            log::info!("{}", out.as_str());
        }

        Timer::after_millis(200).await;
    }
}

#[inline]
pub fn normalize_value(raw: u16, cfg: AdcChanCfg) -> u32 {
    let mut v = if cfg.invert {
        cfg.min.saturating_add(cfg.max).saturating_sub(raw)
    } else {
        raw
    };

    if v < cfg.min {
        v = cfg.min;
    }
    if v > cfg.max {
        v = cfg.max;
    }

    let span = cfg.max.saturating_sub(cfg.min);
    if span == 0 {
        return 0;
    }

    let v0 = (v - cfg.min) as u32;
    ((v0 * 1023) + (span as u32 / 2)) / (span as u32)
}

pub fn read_adc_value(channel: usize) -> u32 {
    if channel < 8 {
        ADC_VALUES[channel].load(Ordering::Relaxed)
    } else {
        0
    }
}
