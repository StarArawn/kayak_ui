#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FloatRGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl FloatRGB {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

#[derive(Debug, Clone)]
pub struct FloatBmp {
    buffer: Vec<f32>,
    w: usize,
    h: usize,
}

impl FloatBmp {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(w * h),
            w,
            h,
        }
    }

    pub fn width(&self) -> usize {
        self.w
    }

    pub fn height(&self) -> usize {
        self.h
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: f32) {
        self.buffer[x + (y * self.w)] = value;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> f32 {
        self.buffer[x + (y * self.w)]
    }
}

#[derive(Debug, Clone)]
pub struct FloatRGBBmp {
    pub buffer: Vec<FloatRGB>,
    w: usize,
    h: usize,
}

impl FloatRGBBmp {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            buffer: vec![FloatRGB::new(0.0, 0.0, 0.0); w * h],
            w,
            h,
        }
    }

    pub fn width(&self) -> usize {
        self.w
    }

    pub fn height(&self) -> usize {
        self.h
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: FloatRGB) {
        self.buffer[x + (y * self.w)] = value;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> FloatRGB {
        self.buffer[x + (y * self.w)]
    }
}
