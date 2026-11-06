use crate::framebuffer::Framebuffer;
use crate::obj::Obj;
use crate::transform::ScreenTransform;
use crate::triangle::triangle;

/// Dibuja el modelo en wireframe: cara → índices → vértices → 2D → triángulo.
pub fn render(fb: &mut Framebuffer, obj: &Obj, transform: &ScreenTransform) {
    for face in &obj.faces {
        // 1. Los tres índices de la cara.
        let [i0, i1, i2] = *face;

        // 2. Los tres vértices del modelo a los que apuntan.
        let v0 = obj.vertices[i0];
        let v1 = obj.vertices[i1];
        let v2 = obj.vertices[i2];

        // 3. Transformación a coordenadas 2D de pantalla.
        let a = transform.apply(v0);
        let b = transform.apply(v1);
        let c = transform.apply(v2);

        // 4. Las tres aristas del triángulo, píxel por píxel.
        triangle(fb, a, b, c);
    }
}
