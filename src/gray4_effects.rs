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
    let top_row = (h as u32).saturating_sub(filled_rows);

    for y in 0..h {
        let srow = src.row(y);
        let drow = dst.row_mut(y);

        gray4::unpack_row_4_to_nibbles(srow, scratch_row, w);

        let in_fill = (y as u32) >= (h as u32).saturating_sub(filled_rows);
        let is_top_row = (y as u32) == top_row;
        let b = if in_fill { fb } else { eb };

        for (x, px) in scratch_row[..w].iter_mut().enumerate() {
            let v = (*px & 0x0F) as usize;

            let mut out = MUL4[b][v];

            if in_fill && is_top_row && *px > 0 {
                out = (out + 2).min(15);
            } else if !in_fill {
                let pattern = ((x as u8 + y as u8) & 1) == 0;
                if pattern {
                    out = out.saturating_sub(1);
                }
            }

            *px = out;
        }

        gray4::pack_row_nibbles_to_4(&scratch_row[..w], drow, w)
    }
}
