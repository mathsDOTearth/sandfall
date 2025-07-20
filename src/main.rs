//! Sand simulation with reusable flat buffer and bottom-centre drain
//! by Rich from mathsDOTearth, 20250720

extern crate minifb;

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use rayon::prelude::*;
use unirand::MarsagliaUniRng;

mod render;
use render::{buffer_to_u32_in_place, draw_pixel, draw_rect, Pixel};

pub const WIDTH: usize = 1200;
pub const HEIGHT: usize = 800;

const SAND: Pixel = Pixel { r: 194, g: 178, b: 128, a: 255 };
const SPAWN_RADIUS: usize = 6;
const TRIES_PER_FRAME: usize = 25;

const DRAIN_X: usize = WIDTH / 2;
const DRAIN_Y: usize = HEIGHT - 1;
const DRAIN_HALF: usize = 50;

#[derive(Clone, Copy)]
struct Grain {
    x: usize,
    y: usize,
}

fn main() {
    let mut window = Window::new("Sand", WIDTH, HEIGHT, WindowOptions::default())
        .expect("Unable to create window");

    let mut pixel_buffer = vec![vec![Pixel::new(0, 0, 0, 255); WIDTH]; HEIGHT];
    let mut flat_buffer = vec![0u32; WIDTH * HEIGHT];

    let mut grid = vec![vec![false; WIDTH]; HEIGHT];
    let mut grains = Vec::<Grain>::new();

    let mut min_x = WIDTH;
    let mut max_x = 0;
    let mut min_y = HEIGHT;
    let mut max_y = 0;

    let mut show_bounds = false;
    let mut last_b_state = false;

    let mut rng = MarsagliaUniRng::new();
    rng.rinit(170);

    let in_bounds = |x: isize, y: isize| {
        x >= 0 && y >= 0 && (x as usize) < WIDTH && (y as usize) < HEIGHT
    };

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let b_down = window.is_key_down(Key::B);
        if b_down && !last_b_state {
            show_bounds = !show_bounds;
        }
        last_b_state = b_down;

        // 1. spawn
        if window.get_mouse_down(MouseButton::Left) {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Discard) {
                let (cx, cy) = (mx as isize, my as isize);
                for _ in 0..TRIES_PER_FRAME {
                    loop {
                        let span = (2 * SPAWN_RADIUS + 1) as f32;
                        let dx = (rng.uni() * span).floor() as isize - SPAWN_RADIUS as isize;
                        let dy = (rng.uni() * span).floor() as isize - SPAWN_RADIUS as isize;
                        if dx * dx + dy * dy > (SPAWN_RADIUS * SPAWN_RADIUS) as isize {
                            continue;
                        }
                        let (x, y) = (cx + dx, cy + dy);
                        if in_bounds(x, y) && !grid[y as usize][x as usize] {
                            let (xu, yu) = (x as usize, y as usize);
                            grid[yu][xu] = true;
                            grains.push(Grain { x: xu, y: yu });

                            if xu < min_x { min_x = xu; }
                            if xu > max_x { max_x = xu; }
                            if yu < min_y { min_y = yu; }
                            if yu > max_y { max_y = yu; }
                        }
                        break;
                    }
                }
            }
        }

        // 2. physics update
        let mut new_min_x = WIDTH;
        let mut new_max_x = 0;
        let mut new_min_y = HEIGHT;
        let mut new_max_y = 0;

        for idx in (0..grains.len()).rev() {
            let Grain { mut x, mut y } = grains[idx];

            if x < min_x || x > max_x || y < min_y || y > max_y {
                continue;
            }

            for (nx, ny) in [
                (x as isize, y as isize + 1),
                (x as isize - 1, y as isize + 1),
                (x as isize + 1, y as isize + 1),
            ] {
                if in_bounds(nx, ny) && !grid[ny as usize][nx as usize] {
                    grid[y][x] = false;
                    x = nx as usize;
                    y = ny as usize;
                    grid[y][x] = true;
                    grains[idx] = Grain { x, y };

                    if x < new_min_x { new_min_x = x; }
                    if x > new_max_x { new_max_x = x; }
                    if y < new_min_y { new_min_y = y; }
                    if y > new_max_y { new_max_y = y; }

                    break;
                }
            }
        }

        if new_min_x <= new_max_x && new_min_y <= new_max_y {
            min_x = new_min_x.saturating_sub(2);
            max_x = (new_max_x + 2).min(WIDTH - 1);
            min_y = new_min_y.saturating_sub(2);
            max_y = (new_max_y + 2).min(HEIGHT - 1);
        }

        // 3. drain
        if window.is_key_down(Key::Space) {
            let start = DRAIN_X.saturating_sub(DRAIN_HALF);
            let end = (DRAIN_X + DRAIN_HALF).min(WIDTH - 1);

            for x in start..=end {
                grid[DRAIN_Y][x] = false;
            }

            grains.retain(|g| {
                let inside = g.y == DRAIN_Y && g.x >= start && g.x <= end;
                if inside {
                    if g.x < min_x { min_x = g.x; }
                    if g.x > max_x { max_x = g.x; }
                    if g.y < min_y { min_y = g.y; }
                    if g.y > max_y { max_y = g.y; }
                }
                !inside
            });
        }

        // 4. clear and draw
        // Parallel clear is now safe!
        pixel_buffer.par_iter_mut().for_each(|row| {
                row.fill(Pixel::new(0, 0, 0, 255));
            });

            // Drawing is kept serial to avoid mutable aliasing
            for g in &grains {
                draw_pixel(&mut pixel_buffer, g.x, g.y, SAND);
            }

        if show_bounds {
            let box_x = min_x as i32;
            let box_y = min_y as i32;
            let box_w = (max_x.saturating_sub(min_x)) as i32;
            let box_h = (max_y.saturating_sub(min_y)) as i32;
            let red = Pixel { r: 255, g: 0, b: 0, a: 255 };
            draw_rect(&mut pixel_buffer, box_x, box_y, box_w, box_h, red);
        }

        buffer_to_u32_in_place(&pixel_buffer, &mut flat_buffer);
        window
            .update_with_buffer(&flat_buffer, WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}
