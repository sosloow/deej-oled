use crate::gray4::{self, MUL4};

/// Compose bottom→top fill into `dst_packed` from `src_packed`.
/// Uses a single `scratch_row` (nibbles) to avoid extra allocations.
pub fn fill_bottom_to_top(
    dst_packed: &mut [u8],
    src_packed: &[u8],
    width: usize,
    height: usize,
    fill_0_1023: u16,       // ADC 0..=1023
    empty_b: u8,            // Gray4 0..=15
    full_b: u8,             // Gray4 0..=15
    scratch_row: &mut [u8], // len >= width (nibbles 0..=15)
) {
    let rb = gray4::row_bytes(width);
    let eb = empty_b.min(15) as usize;
    let fb = full_b.min(15) as usize;

    let filled_rows = ((height as u32 * fill_0_1023 as u32) + 511) / 1023;

    for y in 0..height {
        let srow = &src_packed[y * rb..(y + 1) * rb];
        let drow = &mut dst_packed[y * rb..(y + 1) * rb];

        gray4::unpack_row_4_to_nibbles(srow, scratch_row, width);

        let b = if (y as u32) >= (height as u32).saturating_sub(filled_rows) {
            fb
        } else {
            eb
        };
        for px in &mut scratch_row[..width] {
            let v = (*px & 0x0F) as usize;
            *px = MUL4[b][v];
        }

        gray4::pack_row_nibbles_to_4(&scratch_row[..width], drow, width);
    }
}
