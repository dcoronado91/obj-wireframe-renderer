use raylib::prelude::Vector2;

use crate::framebuffer::Framebuffer;

/// Línea de Bresenham entre dos puntos en pantalla, con el color actual
/// del framebuffer. Cada extremo se redondea al píxel más cercano.
pub fn line(fb: &mut Framebuffer, a: Vector2, b: Vector2) {
    let mut x0 = a.x.round() as i32;
    let mut y0 = a.y.round() as i32;
    let x1 = b.x.round() as i32;
    let y1 = b.y.round() as i32;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        fb.point(x0, y0);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raylib::prelude::Color;

    fn lit_pixels(fb: &Framebuffer) -> Vec<(i32, i32)> {
        let mut pixels = Vec::new();
        for y in 0..fb.height {
            for x in 0..fb.width {
                if fb.get_color(x, y) == Some(Color::WHITE) {
                    pixels.push((x, y));
                }
            }
        }
        pixels
    }

    #[test]
    fn draws_both_endpoints_with_one_pixel_per_major_step() {
        let mut fb = Framebuffer::new(12, 8);
        fb.clear();
        line(&mut fb, Vector2::new(1.0, 1.0), Vector2::new(10.0, 5.0));

        let pixels = lit_pixels(&fb);
        assert_eq!(pixels.len(), 10);
        assert!(pixels.contains(&(1, 1)) && pixels.contains(&(10, 5)));
    }

    #[test]
    fn draws_vertical_and_reversed_lines() {
        let mut fb = Framebuffer::new(5, 5);
        fb.clear();
        line(&mut fb, Vector2::new(2.0, 4.0), Vector2::new(2.0, 0.0));
        assert_eq!(lit_pixels(&fb), (0..5).map(|y| (2, y)).collect::<Vec<_>>());
    }
}
