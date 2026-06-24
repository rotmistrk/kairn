//! Procedural cairn logo — stacked stones with growing plant.
//! Generates a 64×64 RGBA pixel buffer.

/// Generate the cairn logo as RGBA pixels (64×64).
pub(crate) fn generate_cairn_logo() -> (u32, u32, Vec<u8>) {
    let w: u32 = 64;
    let h: u32 = 64;
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    draw_stones(&mut pixels, w);
    draw_plant(&mut pixels, w);
    (w, h, pixels)
}

fn draw_stones(pixels: &mut [u8], w: u32) {
    // Bottom stone: large, dark gray, slightly right
    draw_stone(pixels, w, 34, 56, 22, 10, [90, 85, 80]);
    // Middle stone: medium, mid gray, slightly left
    draw_stone(pixels, w, 30, 44, 16, 9, [120, 115, 108]);
    // Top stone: small, lighter, offset right
    draw_stone(pixels, w, 35, 34, 12, 7, [150, 145, 138]);
    // Tiny cap stone: smallest, lightest
    draw_stone(pixels, w, 32, 26, 8, 5, [175, 170, 162]);
}

fn draw_stone(pixels: &mut [u8], w: u32, cx: i32, cy: i32, rx: i32, ry: i32, base: [u8; 3]) {
    let h = 64i32;
    for y in 0..h {
        for x in 0..w as i32 {
            let dx = (x - cx) as f32 / rx as f32;
            let dy = (y - cy) as f32 / ry as f32;
            let dist = dx * dx + dy * dy;
            if dist <= 1.0 {
                let shade = 1.0 - (dy * 0.3 + 0.15);
                let r = (base[0] as f32 * shade).clamp(0.0, 255.0) as u8;
                let g = (base[1] as f32 * shade).clamp(0.0, 255.0) as u8;
                let b = (base[2] as f32 * shade).clamp(0.0, 255.0) as u8;
                let alpha = if dist > 0.85 {
                    ((1.0 - dist) / 0.15 * 255.0) as u8
                } else {
                    255
                };
                blend_pixel(pixels, w, x as u32, y as u32, [r, g, b, alpha]);
            }
        }
    }
}

fn draw_plant(pixels: &mut [u8], w: u32) {
    // Stem: thin green line growing from top stone
    for y in 8..26 {
        let sway = ((y as f32 - 16.0) * 0.08).sin() * 1.5;
        let x = (33.0 + sway) as u32;
        set_pixel(pixels, w, x, y, [60, 140, 50, 220]);
    }
    // Leaf 1: left, at y=14
    draw_leaf(pixels, w, 31, 14, -1);
    // Leaf 2: right, at y=10
    draw_leaf(pixels, w, 35, 10, 1);
    // Leaf 3: left, at y=7 (top)
    draw_leaf(pixels, w, 32, 7, -1);
}

fn draw_leaf(pixels: &mut [u8], w: u32, cx: u32, cy: u32, dir: i32) {
    // Small teardrop-shaped leaf
    for i in 0..5 {
        let x = (cx as i32 + dir * i) as u32;
        let half = if i < 3 {
            i
        } else {
            5 - i
        } as u32;
        for dy in 0..=half {
            let green = 130 + (i as u8) * 15;
            blend_pixel(pixels, w, x, cy - dy, [40, green, 35, 200]);
            if dy > 0 {
                blend_pixel(pixels, w, x, cy + dy, [40, green, 35, 200]);
            }
        }
    }
}

fn blend_pixel(pixels: &mut [u8], w: u32, x: u32, y: u32, rgba: [u8; 4]) {
    if x >= w || y >= 64 {
        return;
    }
    let idx = ((y * w + x) * 4) as usize;
    let alpha = rgba[3] as f32 / 255.0;
    let inv = 1.0 - alpha;
    pixels[idx] = (rgba[0] as f32 * alpha + pixels[idx] as f32 * inv) as u8;
    pixels[idx + 1] = (rgba[1] as f32 * alpha + pixels[idx + 1] as f32 * inv) as u8;
    pixels[idx + 2] = (rgba[2] as f32 * alpha + pixels[idx + 2] as f32 * inv) as u8;
    pixels[idx + 3] = pixels[idx + 3].saturating_add(rgba[3]);
}

fn set_pixel(pixels: &mut [u8], w: u32, x: u32, y: u32, rgba: [u8; 4]) {
    if x >= w || y >= 64 {
        return;
    }
    let idx = ((y * w + x) * 4) as usize;
    pixels[idx] = rgba[0];
    pixels[idx + 1] = rgba[1];
    pixels[idx + 2] = rgba[2];
    pixels[idx + 3] = rgba[3];
}
