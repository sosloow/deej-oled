use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;
use tinybmp::Bmp;

pub const BUFFER_SIZE: usize = 4096;

pub struct FilteredBmp<'d> {
    original: &'d [u8],
    pub data: [u8; BUFFER_SIZE],
}

impl<'d> FilteredBmp<'d> {
    pub fn new(image_data: &'d [u8]) -> Self {
        assert!(image_data.len() <= BUFFER_SIZE, "Image data is too large!");

        let mut cloned_data = [0u8; BUFFER_SIZE];
        cloned_data[..image_data.len()].copy_from_slice(image_data);

        Self {
            original: &image_data,
            data: cloned_data,
        }
    }

    pub fn dim(&mut self, dim: u8) {
        let dim = dim.min(100);

        let original_bmp: Bmp<'d, Gray4> = Bmp::from_slice(&self.original).unwrap();

        let image_data_start = original_bmp.as_raw().header().image_data_start;

        for i in image_data_start..self.original.len() {
            let byte = self.original[i];

            self.data[i] = (byte as u16 * (100 - dim as u16) / 100) as u8;
        }
    }

    pub fn draw<D>(&self, display: &mut D, coords: Point) -> Result<(), <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = Gray4>,
    {
        let bmp: Bmp<Gray4> = Bmp::from_slice(&self.data).unwrap();
        let image = Image::new(&bmp, coords);

        image.draw(display)
    }
}
