use core::fmt::Write as _;
use core::sync::atomic::Ordering;
use heapless::String;
use static_cell::StaticCell;

use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_usb::{class::cdc_acm, Builder, Config as UsbConfig, UsbDevice};
use embassy_usb_logger::MAX_PACKET_SIZE;

use crate::adc::{ADC_FORCE_PUSH, ADC_VALUES};
use crate::graphics::{ScreenState, SCREEN_STATE};
use crate::{Irqs, UsbResources};

static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
static LOG_STATE: StaticCell<cdc_acm::State> = StaticCell::new();
static USB_DEVICE: StaticCell<UsbDevice<'static, Driver<'static, USB>>> = StaticCell::new();

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HostState {
    Active,
    Suspended,
}

pub static HOST_STATE_CH: Channel<ThreadModeRawMutex, HostState, 1> = Channel::new();

#[embassy_executor::task]
pub async fn usb_task(dev: &'static mut UsbDevice<'static, Driver<'static, USB>>) -> ! {
    let tx = HOST_STATE_CH.sender();

    loop {
        tx.send(HostState::Active).await;
        ADC_FORCE_PUSH.store(true, Ordering::Relaxed);
        SCREEN_STATE.store(ScreenState::INTRO as u8, Ordering::Relaxed);

        dev.run_until_suspend().await;

        tx.send(HostState::Suspended).await;

        dev.wait_resume().await;
    }
}

#[embassy_executor::task]
pub async fn logger_task(class: cdc_acm::CdcAcmClass<'static, Driver<'static, USB>>) {
    let fut = embassy_usb_logger::with_class!(1024, log::LevelFilter::Info, class);
    fut.await;
}

pub fn init(
    res: UsbResources,
) -> (
    &'static mut UsbDevice<'static, Driver<'static, USB>>,
    cdc_acm::CdcAcmClass<'static, Driver<'static, USB>>,
) {
    let driver = Driver::new(res.usb, Irqs);

    let mut config = UsbConfig::new(0xc0de, 0xcafe);
    config.manufacturer = Some("kareraisu.me");
    config.product = Some("deej OLED");
    config.serial_number = Some("oledassfart");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut builder = Builder::new(
        driver,
        config,
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        &mut [], // no MSOS descriptors
        CONTROL_BUF.init([0; 64]),
    );

    let log_class = cdc_acm::CdcAcmClass::new(
        &mut builder,
        LOG_STATE.init(cdc_acm::State::new()),
        MAX_PACKET_SIZE as u16,
    );

    let usb_dev = USB_DEVICE.init(builder.build());

    (usb_dev, log_class)
}

pub fn write_adc_values(values: [u32; ADC_VALUES.len()]) {
    let mut out: String<96> = String::new();

    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            let _ = out.push('|');
        }
        let _ = write!(out, "{}", v);
    }
    log::info!("{}", out.as_str());
}
