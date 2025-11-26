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

pub fn draw_sheet_frame_flash<D: DrawTarget<Color = Gray4>>(
    display: &mut D,
    sheet: &[u8],
    w: u32,
    h: u32,
    idx: usize,
    pos: Point,
    flash_step: u8,  // 0..=flash_steps (0 = biggest flash)
    flash_steps: u8, // duration of flash, e.g. 16
) -> Result<(), D::Error> {
    let frm = frame_slice(sheet, w, h, idx);

    let width = w as usize;
    let height = h as usize;
    let stride = row_bytes(width);

    // If no flash or we've gone past the flash duration,
    // just draw normally at base brightness.
    if flash_steps == 0 || flash_step >= flash_steps {
        return draw_sheet_frame_masked(display, sheet, w, h, idx, pos);
    }

    // Monotonic decay:
    // step = 0           -> boost ≈ MAX_BOOST  (strong flash)
    // step = flash_steps -> boost = 0         (normal brightness, but we return early above)
    const MAX_BOOST: u8 = 6; // tweak to taste

    let steps = flash_steps as u16;
    let step = flash_step as u16;
    let remaining = steps.saturating_sub(step); // steps..1

    // scaled with rounding: boost = MAX_BOOST * remaining / steps
    let boost: u8 = ((remaining * MAX_BOOST as u16 + steps / 2) / steps) as u8;

    for y in 0..height {
        let row_start = y * stride;
        let row = &frm[row_start..row_start + stride];

        let mut x = 0usize;

        for byte in row {
            if x >= width {
                break;
            }

            // high nibble
            let hi = (byte >> 4) & 0x0F;
            if hi != 0 && x < width {
                let mut v = hi;
                v = core::cmp::min(v.saturating_add(boost), 15);

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
                let mut v = lo;
                v = core::cmp::min(v.saturating_add(boost), 15);

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

pub fn draw_sheet_frame_masked_crt<D: DrawTarget<Color = Gray4>>(
    display: &mut D,
    sheet: &[u8],
    w: u32,
    h: u32,
    idx: usize,
    pos: Point,
    frame_tick: u8,
    is_glitching: bool,
) -> Result<(), D::Error> {
    let frm = frame_slice(sheet, w, h, idx);

    let width = w as usize;
    let height = h as usize;
    let stride = row_bytes(width);

    if !is_glitching {
        return draw_sheet_frame_masked(display, sheet, w, h, idx, pos);
    }

    for y in 0..height {
        let row_start = y * stride;
        let row = &frm[row_start..row_start + stride];

        let mut x = 0usize;

        let seed = (y as u8)
            .wrapping_mul(13)
            .wrapping_add(frame_tick.wrapping_mul(7));
        let r = (seed & 0x07) as i8; // 0..7

        let x_offset: i32 = match r {
            0 => -2,
            1 => -1,
            2 => 0,
            3 => 1,
            4 => 2,
            5 => 0,
            6 => -1,
            _ => 1,
        };

        for byte in row {
            if x >= width {
                break;
            }

            let hi = (byte >> 4) & 0x0F;
            if hi != 0 && x < width {
                let mut v = hi;

                // only mild dim
                v = v.saturating_sub(1);

                if v > 0 {
                    let color = Gray4::new(v);
                    let pt = pos + Point::new(x as i32 + x_offset, y as i32);
                    display.draw_iter(core::iter::once(Pixel(pt, color)))?;
                }
            }

            x += 1;
            if x >= width {
                break;
            }

            let lo = byte & 0x0F;
            if lo != 0 && x < width {
                let mut v = lo;

                v = v.saturating_sub(1);

                if v > 0 {
                    let color = Gray4::new(v);
                    let pt = pos + Point::new(x as i32 + x_offset, y as i32);
                    display.draw_iter(core::iter::once(Pixel(pt, color)))?;
                }
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
    fade_step: u8,
    fade_steps: u8,
) -> Result<(), D::Error> {
    if fade_steps == 0 || fade_step == 0 {
        return draw_sheet_frame_masked(display, sheet, w, h, idx, pos);
    }

    let frm = frame_slice(sheet, w, h, idx);

    let width = w as usize;
    let height = h as usize;
    let stride = row_bytes(width);

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

            let dither = ((x as i32 & 1) ^ (y as i32 & 1)) != 0;

            let hi = (byte >> 4) & 0x0F;
            if hi != 0 && x < width {
                let mut v = scale_gray4(hi, num, den);

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
