use embassy_time::{Duration, Ticker};
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;

const FRAME_DELAY: u64 = 20;

use crate::animation_node::AnimationNode;
use crate::screen;

#[embassy_executor::task]
pub async fn prepare_frame_task() {
    let image_data = include_bytes!(sprite_location);

    let bmp: Bmp<Gray4> = Bmp::from_slice(data).unwrap();

    let image = Image::new(&bmp, self.coords);

    let mut animation_root = AnimationNode::new(
        Point::new(83, 0),
        Point::new(87, 10),
        Point::new(screen::SCREEN_WIDTH as i32, screen::SCREEN_HEIGHT as i32),
        Point::new(1, 2),
        100,
        "sprites/parts/outline.bmp",
    );

    let mut ticker = Ticker::every(Duration::from_millis(FRAME_DELAY));

    loop {
        let frame = screen::NEXT_FRAME.wait().await;
        frame.clear(Gray4::BLACK).unwrap();

        animation_root.update();

        animation_root.draw(frame).unwrap();

        screen::READY_FRAME.signal(frame);

        ticker.next().await;
    }
}
