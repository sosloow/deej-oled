// use bumpalo::Bump;
use embassy_time::{Duration, Ticker};
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;

const FRAME_DELAY: u64 = 20;

// use crate::animation_node::BoneBlueprint;
// use crate::animation_node::Skeleton;
// use crate::animation_node::CHILD_COUNT;
use crate::screen;
use crate::volume_indicator::VolumeIndicator;

// const SKELETON: BoneBlueprint = BoneBlueprint {
//     start: Point::new(83, 0),
//     end: Point::new(87, 10),
//     max: Point::new(screen::SCREEN_WIDTH as i32, screen::SCREEN_HEIGHT as i32),
//     freq: Point::new(1, 2),
//     duration: 100,
//     sprite: include_bytes!("sprites/parts/outline.bmp"),
//     children: &[BoneBlueprint {
//         start: Point::new(0, 0),
//         end: Point::new(0, 0),
//         max: Point::new(screen::SCREEN_WIDTH as i32, screen::SCREEN_HEIGHT as i32),
//         freq: Point::new(1, 2),
//         duration: 100,
//         sprite: include_bytes!("sprites/parts/jaw.bmp"),
//         children: &[],
//     }],
// };

#[embassy_executor::task]
pub async fn prepare_frame_task() {
    // let arena = Bump::new();
    // let mut skeleton: Skeleton<'_, Gray4, CHILD_COUNT> =
    //     Skeleton::from_blueprint(&SKELETON, &arena);

    let mut ticker = Ticker::every(Duration::from_millis(FRAME_DELAY));

    let indicator: VolumeIndicator = VolumeIndicator::new(Point::new(5, 15));

    loop {
        let frame = screen::NEXT_FRAME.wait().await;
        frame.clear(Gray4::BLACK).unwrap();

        // skeleton.update();

        // skeleton.draw(frame).ok();

        indicator.draw(frame);

        screen::READY_FRAME.signal(frame);

        ticker.next().await;
    }
}
