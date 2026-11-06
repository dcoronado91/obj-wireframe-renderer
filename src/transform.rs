use raylib::prelude::{Vector2, Vector3};

/// Fracción del lado menor de la ventana que ocupa el modelo.
const FILL: f32 = 0.8;

/// Proyección 2D sin perspectiva: usa solo x e y del vértice, aplica una
/// escala uniforme, centra el modelo en la ventana e invierte y (en
/// pantalla y crece hacia abajo).
pub struct ScreenTransform {
    model_center: Vector2,
    screen_center: Vector2,
    scale: f32,
}

impl ScreenTransform {
    /// Ajusta el bounding box x/y del modelo para que su lado mayor ocupe
    /// ~80% del lado menor de la ventana, centrado.
    pub fn fit(vertices: &[Vector3], width: i32, height: i32) -> Self {
        let mut min = Vector2::new(f32::INFINITY, f32::INFINITY);
        let mut max = Vector2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        for v in vertices {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
        }

        let extent = (max.x - min.x).max(max.y - min.y);
        let scale = if extent > 0.0 {
            FILL * width.min(height) as f32 / extent
        } else {
            1.0
        };
        let model_center = if vertices.is_empty() {
            Vector2::zero()
        } else {
            Vector2::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0)
        };

        ScreenTransform {
            model_center,
            screen_center: Vector2::new(width as f32 / 2.0, height as f32 / 2.0),
            scale,
        }
    }

    pub fn apply(&self, v: Vector3) -> Vector2 {
        Vector2::new(
            self.screen_center.x + (v.x - self.model_center.x) * self.scale,
            self.screen_center.y - (v.y - self.model_center.y) * self.scale,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obj::Obj;

    const WIDTH: i32 = 800;
    const HEIGHT: i32 = 600;

    #[test]
    fn untitled_obj_fits_centered_inside_window() {
        let obj = Obj::load(concat!(env!("CARGO_MANIFEST_DIR"), "/Untitled.obj")).unwrap();
        let transform = ScreenTransform::fit(&obj.vertices, WIDTH, HEIGHT);
        let projected: Vec<Vector2> = obj.vertices.iter().map(|&v| transform.apply(v)).collect();

        let half_box = FILL * WIDTH.min(HEIGHT) as f32 / 2.0 + 0.5;
        for p in &projected {
            assert!(p.x >= 0.0 && p.x < WIDTH as f32 && p.y >= 0.0 && p.y < HEIGHT as f32);
            assert!((p.x - WIDTH as f32 / 2.0).abs() <= half_box, "{p:?}");
            assert!((p.y - HEIGHT as f32 / 2.0).abs() <= half_box, "{p:?}");
        }

        let min_x = projected.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let max_x = projected.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = projected.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_y = projected.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        assert!(((min_x + max_x) / 2.0 - WIDTH as f32 / 2.0).abs() < 0.5);
        assert!(((min_y + max_y) / 2.0 - HEIGHT as f32 / 2.0).abs() < 0.5);
        let largest_side = (max_x - min_x).max(max_y - min_y);
        assert!((largest_side - FILL * HEIGHT as f32).abs() < 0.5);
    }

    #[test]
    fn flips_y_axis() {
        let vertices = [Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 0.0)];
        let transform = ScreenTransform::fit(&vertices, 100, 100);
        let low = transform.apply(vertices[0]);
        let high = transform.apply(vertices[1]);
        assert!(high.y < low.y);
        assert!(high.x > low.x);
    }
}
