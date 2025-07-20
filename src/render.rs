// This module contains rendering helper functions that extend minifb.
// by Rich of maths.earth 202500308

/// A struct to represent an RGBA pixel.
#[derive(Clone, Copy)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    /// Create a new Pixel with the given RGBA values.
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Convert this pixel into a 32-bit colour in 0xAARRGGBB format.
    pub fn to_u32(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

/// Clears the given 2D pixel buffer by filling every pixel with black.
pub fn clear_buffer(buffer: &mut [Vec<Pixel>]) {
    for row in buffer.iter_mut() {
        for pixel in row.iter_mut() {
            *pixel = Pixel::new(0, 0, 0, 255);
        }
    }
}

/// Draw a square into the provided 2D pixel buffer.
/// 
/// * `x` and `y` are the top-left coordinates of the square.
/// * `square_width` and `square_height` specify its dimensions.
/// * `color` is the colour to draw.
pub fn draw_square(
    buffer: &mut [Vec<Pixel>],
    x: usize,
    y: usize,
    square_width: usize,
    square_height: usize,
    color: Pixel,
) {
    for j in y..(y + square_height) {
        for i in x..(x + square_width) {
            // Ensure we remain within bounds.
            if j < buffer.len() && i < buffer[j].len() {
                buffer[j][i] = color;
            }
        }
    }
}

/// Draw a single pixel into the provided 2D pixel buffer.
pub fn draw_pixel(
    buffer: &mut [Vec<Pixel>], 
    x: usize, 
    y: usize, 
    color: 
    Pixel
) {
    if y < buffer.len() && x < buffer[y].len() {
        buffer[y][x] = color;
    }
}

/// Draw a line in to the provided 2D pixel buffer.
pub fn draw_line(
    buffer: &mut [Vec<Pixel>],
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: Pixel,
) {
    // Calculate the differences.
    let delta_x = x1 - x0;
    let delta_y = y1 - y0;
    
    // Determine the number of steps needed based on the longest side.
    let longest_side_length = if delta_x.abs() >= delta_y.abs() {
        delta_x.abs()
    } else {
        delta_y.abs()
    };

    // If the line is just a point, draw that pixel.
    if longest_side_length == 0 {
        draw_pixel(buffer, x0 as usize, y0 as usize, color);
        return;
    }
    
    // Calculate the increments for each step.
    let x_inc = delta_x as f32 / longest_side_length as f32;
    let y_inc = delta_y as f32 / longest_side_length as f32;
    
    // Initialise the current position.
    let mut current_x = x0 as f32;
    let mut current_y = y0 as f32;
    
    // Draw pixels along the line.
    for _ in 0..=longest_side_length {
        let ix = current_x.round() as usize;
        let iy = current_y.round() as usize;
        draw_pixel(buffer, ix, iy, color);
        current_x += x_inc;
        current_y += y_inc;
    }
}

/// Draw a triangle in to the provided 2D pixel buffer.
pub fn draw_triangle(
    buffer: &mut [Vec<Pixel>],
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    color: Pixel,
) {
    draw_line(buffer, x0, y0, x1, y1, color);
    draw_line(buffer, x1, y1, x2, y2, color);
    draw_line(buffer, x2, y2, x0, y0, color);
}

/// Draw a rectangle in to the provided 2D pixel buffer.
pub fn draw_rect(
    buffer: &mut [Vec<Pixel>],
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: Pixel 
) {
    draw_line(buffer, x, y, x + width, y, color);
    draw_line(buffer, x + width, y, x + width, y + height, color);
    draw_line(buffer, x + width, y + height, x, y + height, color);
    draw_line(buffer, x, y + height, x, y, color); 
}

/// Converts a 2D pixel buffer into a 1D vector of u32 values (0xAARRGGBB).
pub fn buffer_to_u32(buffer: &Vec<Vec<Pixel>>) -> Vec<u32> {
    let mut flat: Vec<u32> = Vec::with_capacity(buffer.len() * buffer[0].len());
    for row in buffer {
        for &pixel in row {
            flat.push(pixel.to_u32());
        }
    }
    flat
}

/// Fills `out` with 0xAARRGGBB words converted from `buffer`.
/// `out.len()` **must equal** buffer.len() * buffer[0].len().
pub fn buffer_to_u32_in_place(buffer: &[Vec<Pixel>], out: &mut [u32]) {
    debug_assert_eq!(out.len(), buffer.len() * buffer[0].len());

    let mut i = 0;
    for row in buffer {
        for &pix in row {
            out[i] = pix.to_u32();
            i += 1;
        }
    }
}
