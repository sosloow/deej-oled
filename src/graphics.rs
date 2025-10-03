// use bumpalo::Bump;
use embassy_time::{Duration, Ticker};
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;

const FRAME_DELAY: u64 = 20;

use crate::screen;
use crate::volume_indicator::VolumeIndicator;

#[embassy_executor::task]
pub async fn prepare_frame_task() {
    let mut ticker = Ticker::every(Duration::from_millis(FRAME_DELAY));

    let mut indicator: VolumeIndicator = VolumeIndicator::new(Point::new(98, 3));

    loop {
        let frame = screen::NEXT_FRAME.wait().await;
        frame.clear(Gray4::BLACK).unwrap();

        indicator.draw(frame);

        screen::READY_FRAME.signal(frame);

        ticker.next().await;
    }
}
