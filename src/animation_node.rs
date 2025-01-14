use core::usize;

use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;
use libm::{round, sin};
use tinybmp::Bmp;

const PI: f64 = 3.14159265358979323846;

fn abs_sin(frame: u32, duration: u32, start: Point, end: Point, freq: Point) -> Point {
    let tp: f64 = frame as f64 / duration as f64 * PI;
    let offset = end - start;

    let x = start.x + round(offset.x as f64 * sin(tp * freq.x as f64)) as i32;
    let y = start.y + round(offset.y as f64 * sin(tp * freq.y as f64)) as i32;

    Point::new(x.abs(), y.abs())
}

pub struct AnimationNode<'d> {
    pub max: Point,
    pub coords: Point,
    pub start: Point,
    pub end: Point,
    pub freq: Point,
    pub frame: u32,
    pub duration: u32,
    pub sprite: Image<'d, Bmp<'d, Gray4>>,
}

impl<'d> AnimationNode<'d> {
    pub fn new(
        start: Point,
        end: Point,
        max: Point,
        freq: Point,
        duration: u32,
        sprite: Image<'d, Bmp<'d, Gray4>>,
    ) -> Self {
        Self {
            frame: 1,
            duration,
            coords: Point::new(1, 1),
            start,
            end,
            max,
            freq,
            sprite,
        }
    }

    pub fn update(&mut self) {
        self.coords = abs_sin(self.frame, self.duration, self.start, self.end, self.freq);

        self.frame = if self.frame >= self.duration {
            1
        } else {
            self.frame + 1
        };
    }

    pub fn draw<D>(&self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Gray4>,
    {
        self.sprite.draw(display)
    }
}
