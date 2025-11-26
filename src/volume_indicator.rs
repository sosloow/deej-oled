use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;

use crate::adc::AdcTarget;
use crate::gray4::{self, Gray4Img, Gray4ImgMut};
use crate::gray4_effects::{fill_bottom_to_top, FillParams};

static VOLUME_ICON_SYSTEM: &[u8] = include_bytes!("sprites/logos/system-62.gray4");
static VOLUME_ICON_MIC: &[u8] = include_bytes!("sprites/logos/mic-62.gray4");
static VOLUME_ICON_STEAM: &[u8] = include_bytes!("sprites/logos/steam-62.gray4");
static VOLUME_ICON_DISCORD: &[u8] = include_bytes!("sprites/logos/discord-62.gray4");
static VOLUME_ICON_BROWSER: &[u8] = include_bytes!("sprites/logos/browser-62.gray4");

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

    pub fn draw<D>(&mut self, display: &mut D, adc_value: u16, adc_target: AdcTarget)
    where
        D: DrawTarget<Color = Gray4>,
    {
        let volume_icon = match adc_target {
            AdcTarget::System => VOLUME_ICON_SYSTEM,
            AdcTarget::Mic => VOLUME_ICON_MIC,
            AdcTarget::Browser => VOLUME_ICON_BROWSER,
            AdcTarget::Steam => VOLUME_ICON_STEAM,
            AdcTarget::Discord => VOLUME_ICON_DISCORD,
        };

        let mut dst = Gray4ImgMut {
            bytes: &mut self.out_buf,
            w: W,
            h: H,
        };
        let src = Gray4Img {
            bytes: volume_icon,
            w: W,
            h: H,
        };

        fill_bottom_to_top(
            &mut dst,
            &src,
            adc_value,
            FillParams {
                empty_b: 1,
                full_b: 3,
            },
            &mut self.scratch_row,
        );

        let raw = ImageRawLE::<Gray4>::new(&self.out_buf, W as u32);
        Image::new(&raw, self.coords).draw(display).ok();
    }
}
