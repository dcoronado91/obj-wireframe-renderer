use std::fs;
use std::path::Path;

use raylib::prelude::{Color, Image};

/// Buffer de color propio. Todo el dibujo ocurre aquí, píxel por píxel;
/// raylib solo se usa para convertirlo en imagen (textura o PNG).
pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    pixels: Vec<Color>,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32) -> Self {
        Framebuffer {
            width,
            height,
            pixels: vec![Color::BLACK; (width * height) as usize],
            background_color: Color::BLACK,
            current_color: Color::WHITE,
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn clear(&mut self) {
        self.pixels.fill(self.background_color);
    }

    /// Pinta un píxel con el color actual; fuera de los límites se ignora.
    pub fn point(&mut self, x: i32, y: i32) {
        if let Some(index) = self.index(x, y) {
            self.pixels[index] = self.current_color;
        }
    }

    pub fn get_color(&self, x: i32, y: i32) -> Option<Color> {
        self.index(x, y).map(|index| self.pixels[index])
    }

    /// Píxeles en bytes RGBA8, fila por fila (formato de textura de raylib).
    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|c| [c.r, c.g, c.b, c.a])
            .collect()
    }

    /// Copia el buffer a una `Image` RGBA8 de raylib (para textura o PNG).
    pub fn to_image(&self) -> Image {
        let image = Image::gen_image_color(self.width, self.height, self.background_color);
        let bytes = self.to_rgba_bytes();
        // SAFETY: gen_image_color reserva width * height píxeles RGBA8
        // (4 bytes cada uno), exactamente el tamaño de `bytes`.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), image.data.cast::<u8>(), bytes.len());
        }
        image
    }

    /// Guarda el buffer como PNG, creando la carpeta si hace falta.
    pub fn save_png(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("no se pudo crear {}: {err}", parent.display()))?;
        }
        // ExportImage no reporta errores: se borra el archivo anterior y se
        // comprueba que exista uno nuevo.
        let _ = fs::remove_file(path);
        let path_str = path
            .to_str()
            .ok_or_else(|| format!("ruta no UTF-8: {}", path.display()))?;
        self.to_image().export_image(path_str);
        if path.is_file() {
            Ok(())
        } else {
            Err(format!("raylib no pudo escribir {}", path.display()))
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        let inside = x >= 0 && x < self.width && y >= 0 && y < self.height;
        inside.then(|| (y * self.width + x) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_uses_current_color_and_ignores_out_of_bounds() {
        let mut fb = Framebuffer::new(4, 3);
        fb.set_background_color(Color::BLUE);
        fb.clear();
        fb.set_current_color(Color::RED);
        fb.point(1, 2);
        fb.point(-1, 0);
        fb.point(4, 0);
        fb.point(0, 3);

        assert_eq!(fb.get_color(1, 2), Some(Color::RED));
        assert_eq!(fb.get_color(0, 0), Some(Color::BLUE));
        assert_eq!(fb.get_color(4, 0), None);
    }
}
