use core::fmt::Write;
use embedded_graphics::image::Image;
use embedded_graphics::mono_font::{ascii::FONT_9X15_BOLD, MonoTextStyle};
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use heapless::String;
use tinybmp::Bmp;

use crate::adc::read_adc_value;
use crate::filtered_bmp::FilteredBmp;

static VOLUME_ICON: &[u8] = include_bytes!("sprites/logos/volume.bmp");

static DEFAULT_BRIGHTNESS: Gray4 = Gray4::new(100);

pub struct VolumeIndicator {
    coords: Point,
}

impl VolumeIndicator {
    pub fn new(coords: Point) -> Self {
        Self { coords }
    }

    pub fn draw<D>(&self, display: &mut D)
    where
        D: DrawTarget<Color = Gray4>,
    {
        let adc_value = read_adc_value(0);
        let mut adc_value_string: String<16> = String::new(); // Fixed capacity of 16 chars
        write!(adc_value_string, "{}", adc_value).unwrap();

        let style = MonoTextStyle::new(&FONT_9X15_BOLD, DEFAULT_BRIGHTNESS);
        Text::new(&adc_value_string, self.coords, style)
            .draw(display)
            .ok();

        let mut image = FilteredBmp::new(VOLUME_ICON);
        image.dim((100 - adc_value * 100 / 1024) as u8);

        image.draw(display, self.coords).ok();
    }
}
