use raylib::prelude::Vector2;

use crate::framebuffer::Framebuffer;
use crate::line::line;

/// Triángulo en wireframe: dibuja sus tres aristas con `line`.
pub fn triangle(fb: &mut Framebuffer, a: Vector2, b: Vector2, c: Vector2) {
    line(fb, a, b);
    line(fb, b, c);
    line(fb, c, a);
}
