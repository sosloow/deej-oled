// use bumpalo::Bump;
use embassy_time::{Duration, Ticker};
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;

use crate::sprite::{draw_sheet_frame, frame_count};
use crate::volume_indicator::VolumeIndicator;
use crate::{adc, screen};

const FRAME_DELAY: u64 = 140;

static SPIDER_SHEET: &[u8] = include_bytes!("sprites/muffet.gray4");
// const SPIDER_SHEET_W: u32 = 122;
const SPIDER_SHEET_W: u32 = 104;
const SPIDER_SHEET_H: u32 = 64;

static SPIDER_CLOSE_SHEET: &[u8] = include_bytes!("sprites/muffet_close.gray4");
const SPIDER_CLOSE_SHEET_W: u32 = 122;
const SPIDER_CLOSE_SHEET_H: u32 = 64;

#[embassy_executor::task]
pub async fn prepare_frame_task() {
    let mut ticker = Ticker::every(Duration::from_millis(FRAME_DELAY));

    let mut indicator: VolumeIndicator = VolumeIndicator::new(Point::new(170, 1));
    let mut standby_screen = StandbyScreen::new(
        SPIDER_SHEET,
        Point { x: 0, y: 0 },
        SPIDER_SHEET_W,
        SPIDER_SHEET_H,
        151,
    );
    let mut active_channel_screen = ActiveChannelScreen::new(
        SPIDER_CLOSE_SHEET,
        Point { x: 15, y: 0 },
        SPIDER_CLOSE_SHEET_W,
        SPIDER_CLOSE_SHEET_H,
    );

    loop {
        let frame = screen::NEXT_FRAME.wait().await;
        frame.clear(Gray4::BLACK).unwrap();

        let active_channel = adc::get_active_channel();

        if let Some(idx) = active_channel {
            let adc = adc::read_adc_value(idx) as u16;

            active_channel_screen.draw(frame);
            indicator.draw(frame, adc, adc::ADC_CHANNELS[idx as usize].target);
        } else {
            standby_screen.draw(frame);
        }

        screen::READY_FRAME.signal(frame);

        ticker.next().await;
    }
}

struct StandbyScreen {
    sprite: &'static [u8],
    width: u32,
    coords: Point,
    sprite_w: u32,
    sprite_h: u32,
    frame: usize,
    frame_total: usize,
    direction: bool,
}

impl StandbyScreen {
    pub fn new(
        sprite: &'static [u8],
        coords: Point,
        sprite_w: u32,
        sprite_h: u32,
        width: u32,
    ) -> Self {
        Self {
            sprite,
            coords,
            width,
            sprite_w,
            sprite_h,
            frame: 0,
            frame_total: frame_count(sprite, sprite_w, sprite_h),
            direction: true,
        }
    }

    pub fn draw<D>(&mut self, display: &mut D)
    where
        D: DrawTarget<Color = Gray4>,
    {
        let _ = draw_sheet_frame(
            display,
            self.sprite,
            self.sprite_w,
            self.sprite_h,
            self.frame,
            self.coords,
        );

        self.frame = (self.frame + 1) % self.frame_total;

        if self.direction {
            self.coords += Point::new(1, 0);
        } else {
            self.coords -= Point::new(1, 0);
        }
        if self.coords.x >= self.width as i32 || self.coords.x <= 0 {
            self.direction = !self.direction;
        }
    }
}

struct ActiveChannelScreen {
    sprite: &'static [u8],
    coords: Point,
    sprite_w: u32,
    sprite_h: u32,
    frame: usize,
    frame_total: usize,
}

impl ActiveChannelScreen {
    pub fn new(sprite: &'static [u8], coords: Point, sprite_w: u32, sprite_h: u32) -> Self {
        Self {
            sprite,
            coords,
            sprite_w,
            sprite_h,
            frame: 0,
            frame_total: frame_count(sprite, sprite_w, sprite_h),
        }
    }

    pub fn draw<D>(&mut self, display: &mut D)
    where
        D: DrawTarget<Color = Gray4>,
    {
        let _ = draw_sheet_frame(
            display,
            self.sprite,
            self.sprite_w,
            self.sprite_h,
            self.frame,
            self.coords,
        );

        self.frame = (self.frame + 1) % self.frame_total;
    }
}
