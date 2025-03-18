use bumpalo::Bump;
use core::cell::RefCell;
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::{Rgb555, Rgb565, Rgb888};
use embedded_graphics::prelude::*;
use heapless::Vec;
use libm::{round, sin};
use tinybmp::Bmp;

pub const PI: f64 = 3.14159265358979323846;
pub const CHILD_COUNT: usize = 16;

fn abs_sin(frame: u32, duration: u32, start: Point, end: Point, freq: Point) -> Point {
    let tp: f64 = frame as f64 / duration as f64 * PI;
    let offset = end - start;

    let x = start.x + round(offset.x as f64 * sin(tp * freq.x as f64)) as i32;
    let y = start.y + round(offset.y as f64 * sin(tp * freq.y as f64)) as i32;

    Point::new(x.abs(), y.abs())
}

pub struct BoneBlueprint {
    pub max: Point,
    pub start: Point,
    pub end: Point,
    pub freq: Point,
    pub duration: u32,
    pub sprite: &'static [u8],
    pub children: &'static [BoneBlueprint],
}

pub struct Skeleton<'d, C, const N: usize>
where
    C: PixelColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    pub arena: &'d Bump,
    pub root_bone: &'d RefCell<Bone<'d, C, N>>,
}

impl<'d, C, const N: usize> Skeleton<'d, C, N>
where
    C: PixelColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    pub fn from_blueprint(bone_blueprint: &BoneBlueprint, arena: &'d Bump) -> Self {
        let root_bone = Bone::<C, N>::from_blueprint(bone_blueprint, &arena);

        Self { arena, root_bone }
    }

    pub fn update(&mut self) {
        self.root_bone.borrow_mut().update();
    }

    pub fn draw<D>(&self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        self.root_bone.borrow().draw(display)
    }
}

pub struct Bone<'d, C, const N: usize>
where
    C: PixelColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    pub max: Point,
    pub coords: Point,
    next_coords: Point,
    pub start: Point,
    pub end: Point,
    pub freq: Point,
    pub frame: u32,
    pub duration: u32,
    sprite: Bmp<'d, C>,
    children: Vec<&'d RefCell<Bone<'d, C, N>>, N>,
}

impl<'d, C, const N: usize> Bone<'d, C, N>
where
    C: PixelColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    pub fn from_blueprint(bone_blueprint: &BoneBlueprint, arena: &'d Bump) -> &'d RefCell<Self> {
        let children = bone_blueprint
            .children
            .iter()
            .map(|child_bp| Bone::from_blueprint(child_bp, arena))
            .collect::<heapless::Vec<_, N>>();

        let bone = Self {
            frame: 1,
            duration: bone_blueprint.duration,
            coords: bone_blueprint.start.clone(),
            next_coords: bone_blueprint.start.clone(),
            start: bone_blueprint.start,
            end: bone_blueprint.end,
            max: bone_blueprint.max,
            freq: bone_blueprint.freq,
            sprite: Bmp::from_slice(bone_blueprint.sprite).unwrap(),
            children,
        };

        arena.alloc(RefCell::new(bone))
    }

    pub fn update(&mut self) {
        self.next_coords = abs_sin(self.frame, self.duration, self.start, self.end, self.freq);

        self.frame = if self.frame >= self.duration {
            1
        } else {
            self.frame + 1
        };

        self.coords = self.next_coords;

        for child_bone in &self.children {
            child_bone.borrow_mut().update();
        }
    }

    pub fn draw<D>(&self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let image = Image::new(&self.sprite, self.coords);

        for child_bone in &self.children {
            child_bone.borrow().draw(display).ok();
        }

        image.draw(display)
    }
}
