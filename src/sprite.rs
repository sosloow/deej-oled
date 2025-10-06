use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;

#[inline]
const fn row_bytes(w: u32) -> usize {
    (w as usize + 1) / 2
}

#[inline]
pub fn frame_count(sheet: &[u8], w: u32, h: u32) -> usize {
    let fsz = row_bytes(w) * h as usize;
    sheet.len() / fsz
}

#[inline]
fn frame_slice(sheet: &[u8], w: u32, h: u32, idx: usize) -> &[u8] {
    let fsz = row_bytes(w) * h as usize;
    let start = idx * fsz;
    &sheet[start..start + fsz]
}

pub fn draw_sheet_frame<D: DrawTarget<Color = Gray4>>(
    display: &mut D,
    sheet: &[u8],
    w: u32,
    h: u32,
    idx: usize,
    pos: Point,
) -> Result<(), D::Error> {
    let frm = frame_slice(sheet, w, h, idx);
    let raw = ImageRawLE::<Gray4>::new(frm, w);
    Image::new(&raw, pos).draw(display)
}
