#![allow(dead_code)]

use core::ops::{Deref, DerefMut};

#[inline]
pub const fn size_bytes(width: usize, height: usize) -> usize {
    row_bytes(width) * height
}

#[inline]
pub const fn row_bytes(width: usize) -> usize {
    width.div_ceil(2)
}

pub fn pack_row_8_to_4(src8: &[u8], dst4: &mut [u8], width: usize) {
    debug_assert!(src8.len() >= width);
    debug_assert!(dst4.len() >= row_bytes(width));

    let mut si = 0usize;
    let mut di = 0usize;
    while si + 1 < width {
        let lo = (src8[si] >> 4) & 0x0F;
        let hi = (src8[si + 1] >> 4) & 0x0F;
        dst4[di] = (hi << 4) | lo;
        si += 2;
        di += 1;
    }
    if si < width {
        dst4[di] = (src8[si] >> 4) & 0x0F;
    }
}

pub fn unpack_row_4_to_8(src4: &[u8], dst8: &mut [u8], width: usize) {
    debug_assert!(dst8.len() >= width);
    let mut x = 0usize;
    for &b in src4.iter().take(row_bytes(width)) {
        if x < width {
            dst8[x] = (b & 0x0F) * 17;
            x += 1;
        }
        if x < width {
            dst8[x] = ((b >> 4) & 0x0F) * 17;
            x += 1;
        }
    }
}

pub fn pack_image_8_to_4(src8: &[u8], dst4: &mut [u8], w: usize, h: usize) {
    debug_assert!(src8.len() >= w * h);
    debug_assert!(dst4.len() >= size_bytes(w, h));
    let rb = row_bytes(w);
    for y in 0..h {
        let s = &src8[y * w..(y + 1) * w];
        let d = &mut dst4[y * rb..(y + 1) * rb];
        pack_row_8_to_4(s, d, w);
    }
}

pub fn unpack_image_4_to_8(src4: &[u8], dst8: &mut [u8], w: usize, h: usize) {
    debug_assert!(src4.len() >= size_bytes(w, h));
    debug_assert!(dst8.len() >= w * h);
    let rb = row_bytes(w);
    for y in 0..h {
        let s = &src4[y * rb..(y + 1) * rb];
        let d = &mut dst8[y * w..(y + 1) * w];
        unpack_row_4_to_8(s, d, w);
    }
}

pub fn unpack_row_4_to_nibbles(src_row: &[u8], out_nibbles: &mut [u8], width: usize) {
    debug_assert!(out_nibbles.len() >= width);
    let mut x = 0usize;
    for &b in src_row.iter().take(row_bytes(width)) {
        if x < width {
            out_nibbles[x] = b & 0x0F;
            x += 1;
        }
        if x < width {
            out_nibbles[x] = (b >> 4) & 0x0F;
            x += 1;
        }
    }
}

pub fn pack_row_nibbles_to_4(nibbles: &[u8], dst_row: &mut [u8], width: usize) {
    let mut di = 0usize;
    let mut x = 0usize;
    while x + 1 < width {
        let lo = nibbles[x] & 0x0F;
        let hi = nibbles[x + 1] & 0x0F;
        dst_row[di] = (hi << 4) | lo;
        di += 1;
        x += 2;
    }
    if x < width {
        dst_row[di] = nibbles[x] & 0x0F;
    }
}

pub struct Gray4Img<'a> {
    pub bytes: &'a [u8],
    pub w: usize,
    pub h: usize,
}
pub struct Gray4ImgMut<'a> {
    pub bytes: &'a mut [u8],
    pub w: usize,
    pub h: usize,
}

impl Gray4Img<'_> {
    #[inline]
    pub fn row(&self, y: usize) -> &[u8] {
        &self.bytes[y * row_bytes(self.w)..(y + 1) * row_bytes(self.w)]
    }
}
impl Gray4ImgMut<'_> {
    #[inline]
    pub fn row_mut(&mut self, y: usize) -> &mut [u8] {
        let rb = row_bytes(self.w);
        &mut self.bytes[y * rb..(y + 1) * rb]
    }
}

pub struct Gray4ViewMut<'a> {
    data: &'a mut [u8],
    w: usize,
    h: usize,
}
pub struct Gray4View<'a> {
    data: &'a [u8],
    w: usize,
    h: usize,
}

impl<'a> Gray4ViewMut<'a> {
    pub fn new(data: &'a mut [u8], w: usize, h: usize) -> Self {
        Self { data, w, h }
    }
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        let i = y * row_bytes(self.w) + (x >> 1);
        let b = self.data[i];
        if (x & 1) == 0 {
            b & 0x0F
        } else {
            (b >> 4) & 0x0F
        }
    }
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, v: u8) {
        let i = y * row_bytes(self.w) + (x >> 1);
        let v = v & 0x0F;
        let r = &mut self.data[i];
        if (x & 1) == 0 {
            *r = (*r & 0xF0) | v;
        } else {
            *r = (*r & 0x0F) | (v << 4);
        }
    }
    #[inline]
    pub fn as_ro(&self) -> Gray4View<'_> {
        Gray4View {
            data: self.data,
            w: self.w,
            h: self.h,
        }
    }
}
impl<'a> Gray4View<'a> {
    pub fn new(data: &'a [u8], w: usize, h: usize) -> Self {
        Self { data, w, h }
    }
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        let i = y * row_bytes(self.w) + (x >> 1);
        let b = self.data[i];
        if (x & 1) == 0 {
            b & 0x0F
        } else {
            (b >> 4) & 0x0F
        }
    }
    #[inline]
    pub fn dims(&self) -> (usize, usize) {
        (self.w, self.h)
    }
}

const fn build_mul4_lut() -> [[u8; 16]; 16] {
    let mut lut = [[0u8; 16]; 16];
    let mut b = 0;
    while b < 16 {
        let mut v = 0;
        while v < 16 {
            lut[b][v] = ((v * b + 7) / 15) as u8;
            v += 1;
        }
        b += 1;
    }
    lut
}
pub static MUL4: [[u8; 16]; 16] = build_mul4_lut();

pub struct PackedRows<'a> {
    data: &'a mut [u8],
    w: usize,
    h: usize,
}
impl<'a> PackedRows<'a> {
    pub fn new(data: &'a mut [u8], w: usize, h: usize) -> Self {
        Self { data, w, h }
    }
    pub fn row_mut(&mut self, y: usize) -> &mut [u8] {
        let rb = row_bytes(self.w);
        &mut self.data[y * rb..(y + 1) * rb]
    }
    pub fn row(&self, y: usize) -> &[u8] {
        let rb = row_bytes(self.w);
        &self.data[y * rb..(y + 1) * rb]
    }
}
impl Deref for Gray4ViewMut<'_> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.data
    }
}
impl DerefMut for Gray4ViewMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}
