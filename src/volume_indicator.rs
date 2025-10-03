use core::fmt::Write;
// use embedded_graphics::image::Image;
use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;
use heapless::String;

use crate::adc::read_adc_value;
use crate::gray4;
use crate::gray4_effects::fill_bottom_to_top;

static VOLUME_ICON: &[u8] = include_bytes!("sprites/logos/steam-62.gray4");
const W: usize = 62;
const H: usize = 62;
const BYTES: usize = gray4::size_bytes(W, H);

pub struct VolumeIndicator {
    coords: Point,
    out_buf: [u8; BYTES],
    scratch_row: [u8; W],
}

impl VolumeIndicator {
    pub fn new(coords: Point) -> Self {
        Self {
            coords,
            out_buf: [0; BYTES],
            scratch_row: [0; W],
        }
    }

    pub fn draw<D>(&mut self, display: &mut D)
    where
        D: DrawTarget<Color = Gray4>,
    {
        let adc_value = read_adc_value(0);
        let mut adc_value_string: String<16> = String::new();
        write!(adc_value_string, "{}", adc_value).unwrap();

        // let style = MonoTextStyle::new(&FONT_9X15_BOLD, DEFAULT_BRIGHTNESS);
        // Text::new(&adc_value_string, self.coords, style)
        //     .draw(display)
        //     .ok();

        let adc = read_adc_value(1) as u16;

        fill_bottom_to_top(
            &mut self.out_buf,
            VOLUME_ICON,
            W,
            H,
            adc,
            1,
            3,
            &mut self.scratch_row,
        );

        let raw = ImageRawLE::<Gray4>::new(&self.out_buf, W as u32);
        Image::new(&raw, self.coords).draw(display).ok();
    }
}
