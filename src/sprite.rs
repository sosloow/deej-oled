use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;
use embedded_graphics::Pixel;

use crate::gray4::row_bytes;

#[inline]
pub fn frame_count(sheet: &[u8], w: u32, h: u32) -> usize {
    let fsz = row_bytes(w as usize) * h as usize;
    sheet.len() / fsz
}

#[inline]
fn frame_slice(sheet: &[u8], w: u32, h: u32, idx: usize) -> &[u8] {
    let fsz = row_bytes(w as usize) * h as usize;
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

    let raw = ImageRawLE::<Gray4>::new(&frm, w);
    Image::new(&raw, pos).draw(display)
}

pub fn draw_sheet_frame_masked<D: DrawTarget<Color = Gray4>>(
    display: &mut D,
    sheet: &[u8],
    w: u32,
    h: u32,
    idx: usize,
    pos: Point,
) -> Result<(), D::Error> {
    let frm = frame_slice(sheet, w, h, idx);

    let width = w as usize;
    let height = h as usize;
    let stride = row_bytes(width); // bytes per row

    for y in 0..height {
        let row_start = y * stride;
        let row = &frm[row_start..row_start + stride];

        let mut x = 0usize;

        for byte in row {
            if x >= width {
                break;
            }

            // high nibble = first pixel
            let hi = (byte >> 4) & 0x0F;
            if hi != 0 && x < width {
                let color = Gray4::new(hi);
                let pt = pos + Point::new(x as i32, y as i32);
                display.draw_iter(core::iter::once(Pixel(pt, color)))?;
            }
            x += 1;
            if x >= width {
                break;
            }

            // low nibble = second pixel
            let lo = byte & 0x0F;
            if lo != 0 && x < width {
                let color = Gray4::new(lo);
                let pt = pos + Point::new(x as i32, y as i32);
                display.draw_iter(core::iter::once(Pixel(pt, color)))?;
            }
            x += 1;
        }
    }

    Ok(())
}

fn scale_gray4(v: u8, num: u16, den: u16) -> u8 {
    if v == 0 || num == 0 {
        return 0;
    }

    let v16 = v as u16;
    let scaled = (v16 * num + den / 2) / den; // simple rounding
    core::cmp::min(scaled as u8, 15)
}

pub fn draw_sheet_frame_fade_dither<D: DrawTarget<Color = Gray4>>(
    display: &mut D,
    sheet: &[u8],
    w: u32,
    h: u32,
    idx: usize,
    pos: Point,
    fade_step: u8,  // current fade step (0..fade_steps)
    fade_steps: u8, // total steps until fully black, e.g. 32
) -> Result<(), D::Error> {
    // Safety: if fade_steps is 0, or we’re at step 0, just draw normally
    if fade_steps == 0 || fade_step == 0 {
        return draw_sheet_frame_masked(display, sheet, w, h, idx, pos);
    }

    let frm = frame_slice(sheet, w, h, idx);

    let width = w as usize;
    let height = h as usize;
    let stride = row_bytes(width);

    // Remaining "brightness fraction"
    let clamped_step = core::cmp::min(fade_step, fade_steps) as u16;
    let num = (fade_steps as u16).saturating_sub(clamped_step); // remaining
    let den = fade_steps as u16;

    for y in 0..height {
        let row_start = y * stride;
        let row = &frm[row_start..row_start + stride];

        let mut x = 0usize;

        for byte in row {
            if x >= width {
                break;
            }

            // tiny 2×2 pattern
            let dither = ((x as i32 & 1) ^ (y as i32 & 1)) != 0;

            // high nibble
            let hi = (byte >> 4) & 0x0F;
            if hi != 0 && x < width {
                let mut v = scale_gray4(hi, num, den);

                // nudge some pixels slightly faster near the end of the fade
                if dither && v > 0 && clamped_step > (fade_steps as u16 / 3) {
                    v = v.saturating_sub(1);
                }

                if v > 0 {
                    let color = Gray4::new(v);
                    let pt = pos + Point::new(x as i32, y as i32);
                    display.draw_iter(core::iter::once(Pixel(pt, color)))?;
                }
            }

            x += 1;
            if x >= width {
                break;
            }

            // low nibble
            let lo = byte & 0x0F;
            if lo != 0 && x < width {
                let mut v = scale_gray4(lo, num, den);

                if dither && v > 0 && clamped_step > (fade_steps as u16 / 3) {
                    v = v.saturating_sub(1);
                }

                if v > 0 {
                    let color = Gray4::new(v);
                    let pt = pos + Point::new(x as i32, y as i32);
                    display.draw_iter(core::iter::once(Pixel(pt, color)))?;
                }
            }

            x += 1;
        }
    }

    Ok(())
}
