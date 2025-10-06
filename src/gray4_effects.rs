use crate::gray4::{self, Gray4Img, Gray4ImgMut, MUL4};

pub struct FillParams {
    pub empty_b: u8,
    pub full_b: u8,
}

pub fn fill_bottom_to_top(
    dst: &mut Gray4ImgMut,
    src: &Gray4Img,
    fill_0_1023: u16,
    p: FillParams,
    scratch_row: &mut [u8],
) {
    debug_assert_eq!(dst.w, src.w);
    debug_assert_eq!(dst.h, src.h);
    debug_assert!(scratch_row.len() >= src.w);

    let w = dst.w;
    let h = dst.h;

    let eb = p.empty_b.min(15) as usize;
    let fb = p.full_b.min(15) as usize;

    let filled_rows = ((h as u32 * fill_0_1023 as u32) + 511) / 1023;

    for y in 0..h {
        let srow = src.row(y);
        let drow = dst.row_mut(y);

        gray4::unpack_row_4_to_nibbles(srow, scratch_row, w);

        let b = if (y as u32) >= (h as u32).saturating_sub(filled_rows) {
            fb
        } else {
            eb
        };
        for px in &mut scratch_row[..w] {
            let v = (*px & 0x0F) as usize;
            *px = MUL4[b][v];
        }

        gray4::pack_row_nibbles_to_4(&scratch_row[..w], drow, w)
    }
}
