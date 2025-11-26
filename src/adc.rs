use adc_mcp3008::{self, Channels8, Mcp3008};
use core::sync::atomic::{AtomicBool, AtomicI8, AtomicU32, Ordering};
use embassy_rp::{gpio, peripherals, spi};
use embassy_time::Timer;
use embassy_time::{Duration, Instant};

use crate::{deej_usb, AdcResources};

#[derive(Clone, Copy)]
pub struct AdcChanCfg {
    pub invert: bool,
    pub min: u16,
    pub max: u16,
    pub chan: Channels8,
    pub target: AdcTarget,
}

#[derive(Clone, Copy)]
pub enum AdcTarget {
    System,
    Mic,
    Browser,
    Steam,
    Discord,
    // Spotify,
}

pub const NOISE_THRESHOLD: u32 = 15;

pub const ADC_CHANNELS: [AdcChanCfg; 5] = [
    AdcChanCfg {
        invert: true,
        min: 0,
        max: 1023,
        chan: Channels8::CH0,
        target: AdcTarget::System,
    },
    AdcChanCfg {
        invert: true,
        min: 0,
        max: 1023,
        chan: Channels8::CH1,
        target: AdcTarget::Mic,
    },
    AdcChanCfg {
        invert: true,
        min: 0,
        max: 1023,
        chan: Channels8::CH2,
        target: AdcTarget::Browser,
    },
    AdcChanCfg {
        invert: true,
        min: 0,
        max: 1023,
        chan: Channels8::CH3,
        target: AdcTarget::Steam,
    },
    AdcChanCfg {
        invert: true,
        min: 0,
        max: 1023,
        chan: Channels8::CH4,
        target: AdcTarget::Discord,
    },
];

pub const ACTIVE_CHANNEL_TTL: u32 = 1000;
pub static ACTIVE_CHANNEL: AtomicI8 = AtomicI8::new(-1);

pub static ADC_VALUES: [AtomicU32; 5] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

pub static ADC_FORCE_PUSH: AtomicBool = AtomicBool::new(false);

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

    let mut active_deadline = Instant::now();
    ACTIVE_CHANNEL.store(-1, Ordering::Relaxed);

    loop {
        let mut snapshot: [u32; ADC_VALUES.len()] = [0; ADC_VALUES.len()];
        for (i, s) in snapshot.iter_mut().enumerate() {
            *s = ADC_VALUES[i].load(Ordering::Relaxed);
        }

        let mut any_updated = false;
        let mut best_idx: Option<usize> = None;
        let mut best_diff: u32 = 0;

        for (i, conf) in ADC_CHANNELS.iter().enumerate() {
            if let Ok(raw) = adc_mcp.adc.read_channel(conf.chan) {
                let norm = normalize_value(raw, *conf);
                let curr = snapshot[i];
                let diff = core::cmp::max(curr, norm) - core::cmp::min(curr, norm);

                if diff >= NOISE_THRESHOLD {
                    ADC_VALUES[i].store(norm, Ordering::Relaxed);
                    snapshot[i] = norm;
                    any_updated = true;

                    if diff > best_diff {
                        best_diff = diff;
                        best_idx = Some(i);
                    }
                }
            } else {
                log::warn!("Failed to read channel {}", i);
            }
        }

        let now = Instant::now();

        if let Some(idx) = best_idx {
            set_active_channel(Some(idx));
            active_deadline = now + Duration::from_millis(ACTIVE_CHANNEL_TTL as u64);
        } else if now >= active_deadline {
            set_active_channel(None);
        }

        if any_updated || ADC_FORCE_PUSH.load(Ordering::Relaxed) {
            deej_usb::write_adc_values(snapshot);
            ADC_FORCE_PUSH.store(false, Ordering::Relaxed);
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

fn set_active_channel(channel: Option<usize>) {
    let idx = match channel {
        Some(c) if c < ADC_CHANNELS.len() => c as i8,
        _ => -1,
    };

    ACTIVE_CHANNEL.store(idx, Ordering::Relaxed);
}

pub fn get_active_channel() -> Option<usize> {
    let active_channel = ACTIVE_CHANNEL.load(Ordering::Relaxed);

    if active_channel >= 0 {
        Some(active_channel as usize)
    } else {
        None
    }
}
