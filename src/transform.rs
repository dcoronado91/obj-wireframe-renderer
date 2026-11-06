use raylib::prelude::{Vector2, Vector3};

/// Fracción del lado menor de la ventana que ocupa el modelo.
const FILL: f32 = 0.8;

/// Lado desde el que se mira el modelo (y hacia arriba, +z hacia el frente).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}

impl View {
    pub const ALL: [View; 6] = [
        View::Front,
        View::Back,
        View::Left,
        View::Right,
        View::Top,
        View::Bottom,
    ];

    pub fn name(self) -> &'static str {
        match self {
            View::Front => "front",
            View::Back => "back",
            View::Left => "left",
            View::Right => "right",
            View::Top => "top",
            View::Bottom => "bottom",
        }
    }

    /// Descarta el eje de profundidad y devuelve (derecha, arriba) del
    /// vértice visto desde este lado. La vista frontal usa solo x e y.
    fn project(self, v: Vector3) -> Vector2 {
        match self {
            View::Front => Vector2::new(v.x, v.y),
            View::Back => Vector2::new(-v.x, v.y),
            View::Left => Vector2::new(v.z, v.y),
            View::Right => Vector2::new(-v.z, v.y),
            View::Top => Vector2::new(v.x, -v.z),
            View::Bottom => Vector2::new(v.x, v.z),
        }
    }
}

/// Proyección 2D sin perspectiva: toma dos ejes del vértice según la vista,
/// aplica una escala uniforme, centra el modelo en la ventana e invierte y
/// (en pantalla y crece hacia abajo).
pub struct ScreenTransform {
    view: View,
    model_center: Vector2,
    screen_center: Vector2,
    scale: f32,
}

impl ScreenTransform {
    /// Ajusta el bounding box 2D del modelo para que su lado mayor ocupe
    /// ~80% del lado menor de la ventana, centrado.
    pub fn fit(vertices: &[Vector3], view: View, width: i32, height: i32) -> Self {
        let mut min = Vector2::new(f32::INFINITY, f32::INFINITY);
        let mut max = Vector2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        for &v in vertices {
            let p = view.project(v);
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
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
            view,
            model_center,
            screen_center: Vector2::new(width as f32 / 2.0, height as f32 / 2.0),
            scale,
        }
    }

    pub fn apply(&self, v: Vector3) -> Vector2 {
        let p = self.view.project(v);
        Vector2::new(
            self.screen_center.x + (p.x - self.model_center.x) * self.scale,
            self.screen_center.y - (p.y - self.model_center.y) * self.scale,
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
    fn untitled_obj_fits_centered_inside_window_from_every_view() {
        let obj = Obj::load(concat!(env!("CARGO_MANIFEST_DIR"), "/Untitled.obj")).unwrap();

        for view in View::ALL {
            let transform = ScreenTransform::fit(&obj.vertices, view, WIDTH, HEIGHT);
            let projected: Vec<Vector2> =
                obj.vertices.iter().map(|&v| transform.apply(v)).collect();

            let half_box = FILL * WIDTH.min(HEIGHT) as f32 / 2.0 + 0.5;
            for p in &projected {
                assert!(p.x >= 0.0 && p.x < WIDTH as f32 && p.y >= 0.0 && p.y < HEIGHT as f32);
                assert!((p.x - WIDTH as f32 / 2.0).abs() <= half_box, "{view:?} {p:?}");
                assert!((p.y - HEIGHT as f32 / 2.0).abs() <= half_box, "{view:?} {p:?}");
            }

            let min_x = projected.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
            let max_x = projected.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
            let min_y = projected.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
            let max_y = projected.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
            assert!(((min_x + max_x) / 2.0 - WIDTH as f32 / 2.0).abs() < 0.5, "{view:?}");
            assert!(((min_y + max_y) / 2.0 - HEIGHT as f32 / 2.0).abs() < 0.5, "{view:?}");
            let largest_side = (max_x - min_x).max(max_y - min_y);
            assert!((largest_side - FILL * HEIGHT as f32).abs() < 0.5, "{view:?}");
        }
    }

    #[test]
    fn front_view_flips_y_axis() {
        let vertices = [Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 0.0)];
        let transform = ScreenTransform::fit(&vertices, View::Front, 100, 100);
        let low = transform.apply(vertices[0]);
        let high = transform.apply(vertices[1]);
        assert!(high.y < low.y);
        assert!(high.x > low.x);
    }

    #[test]
    fn opposite_views_mirror_each_other() {
        let vertices = [Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0)];
        let screen = |view| {
            let transform = ScreenTransform::fit(&vertices, view, 100, 100);
            transform.apply(vertices[1])
        };
        assert!(screen(View::Front).x > 50.0 && screen(View::Back).x < 50.0);
        assert!(screen(View::Left).x > 50.0 && screen(View::Right).x < 50.0);
        assert!(screen(View::Top).y > 50.0 && screen(View::Bottom).y < 50.0);
    }
}
