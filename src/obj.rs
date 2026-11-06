use std::fs;
use std::path::Path;

use raylib::prelude::Vector3;

/// Modelo OBJ mínimo: posiciones de vértices y caras trianguladas como
/// índices (base 0) a `vertices`. Solo se leen las líneas `v` y `f`; `vt`,
/// `vn`, materiales y grupos se ignoran.
pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub faces: Vec<[usize; 3]>,
}

impl Obj {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .map_err(|err| format!("no se pudo leer {}: {err}", path.display()))?;
        Self::parse(&source).map_err(|err| format!("{}: {err}", path.display()))
    }

    pub fn parse(source: &str) -> Result<Self, String> {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        for (number, line) in source.lines().enumerate() {
            let line_number = number + 1;
            let mut tokens = line.split_whitespace();
            match tokens.next() {
                Some("v") => {
                    let mut coords = [0.0f32; 3];
                    for coord in &mut coords {
                        *coord = tokens
                            .next()
                            .and_then(|token| token.parse().ok())
                            .ok_or_else(|| format!("línea {line_number}: vértice inválido"))?;
                    }
                    vertices.push(Vector3::new(coords[0], coords[1], coords[2]));
                }
                Some("f") => {
                    let polygon = tokens
                        .map(|token| resolve_index(token, vertices.len()))
                        .collect::<Result<Vec<usize>, String>>()
                        .map_err(|err| format!("línea {line_number}: {err}"))?;
                    if polygon.len() < 3 {
                        return Err(format!("línea {line_number}: una cara necesita 3 vértices"));
                    }
                    // Triangulación en abanico desde el primer vértice.
                    for i in 1..polygon.len() - 1 {
                        faces.push([polygon[0], polygon[i], polygon[i + 1]]);
                    }
                }
                _ => {}
            }
        }

        Ok(Obj { vertices, faces })
    }
}

/// Convierte un token de cara (`7`, `7/1`, `7//2`, `7/1/2`, `-1`) en un
/// índice base 0 a los vértices leídos hasta ahora.
fn resolve_index(token: &str, vertex_count: usize) -> Result<usize, String> {
    let index: i64 = token
        .split('/')
        .next()
        .and_then(|first| first.parse().ok())
        .ok_or_else(|| format!("índice de cara inválido '{token}'"))?;
    let resolved = if index > 0 {
        index - 1
    } else {
        vertex_count as i64 + index
    };
    if index != 0 && (0..vertex_count as i64).contains(&resolved) {
        Ok(resolved as usize)
    } else {
        Err(format!(
            "índice '{token}' fuera de rango (hay {vertex_count} vértices)"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/Untitled.obj");

    #[test]
    fn loads_untitled_obj() {
        let obj = Obj::load(MODEL_PATH).unwrap();
        println!(
            "Untitled.obj: {} vértices, {} triángulos",
            obj.vertices.len(),
            obj.faces.len()
        );
        assert_eq!(obj.vertices.len(), 356);
        assert_eq!(obj.faces.len(), 933);
        assert!(obj.faces.iter().flatten().all(|&i| i < obj.vertices.len()));
    }

    #[test]
    fn parses_slash_formats_and_negative_indices() {
        let source = "\
v 0 0 0
v 1 0 0
v 1 1 0
vt 0 0
vn 0 0 1
f 1 2 3
f 1/1 2/1 3/1
f 1//1 2//1 3//1
f 1/1/1 2/1/1 3/1/1
f -3 -2 -1
";
        let obj = Obj::parse(source).unwrap();
        assert_eq!(obj.vertices.len(), 3);
        assert_eq!(obj.faces, vec![[0, 1, 2]; 5]);
    }

    #[test]
    fn triangulates_quads_and_polygons_as_fan() {
        let source = "\
v 0 0 0
v 1 0 0
v 1 1 0
v 0 1 0
v -1 1 0
f 1/1/1 2/2/1 3/3/1 4/4/1
f 1 2 3 4 5
";
        let obj = Obj::parse(source).unwrap();
        assert_eq!(
            obj.faces,
            vec![[0, 1, 2], [0, 2, 3], [0, 1, 2], [0, 2, 3], [0, 3, 4]]
        );
    }

    #[test]
    fn rejects_out_of_range_indices() {
        let err = Obj::parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 4\n").err().unwrap();
        assert!(err.contains("línea 4") && err.contains("fuera de rango"), "{err}");
        assert!(Obj::parse("v 0 0 0\nf 0 1 1\n").is_err());
        assert!(Obj::parse("v 0 0 0\nf -2 1 1\n").is_err());
    }

    #[test]
    fn reports_missing_file() {
        let err = Obj::load("no_existe.obj").err().unwrap();
        assert!(err.contains("no se pudo leer no_existe.obj"), "{err}");
    }
}
