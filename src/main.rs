use std::path::{Path, PathBuf};

use raylib::prelude::*;

use obj_wireframe_renderer::framebuffer::Framebuffer;
use obj_wireframe_renderer::obj::Obj;
use obj_wireframe_renderer::render::render;
use obj_wireframe_renderer::transform::{ScreenTransform, View};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const BACKGROUND: Color = Color::new(18, 20, 28, 255);
const MODEL_COLOR: Color = Color::new(225, 235, 245, 255);
const MODEL_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/Untitled.obj");
const EVIDENCE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/evidence");
const USAGE: &str = "uso: obj_wireframe_renderer [--screenshot]";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => run_window(),
        [flag] if flag == "--screenshot" => screenshot(),
        _ => {
            eprintln!("argumentos no reconocidos: {}\n{USAGE}", args.join(" "));
            std::process::exit(2);
        }
    }
}

fn load_model() -> Obj {
    let obj = Obj::load(MODEL_PATH).unwrap_or_else(|err| {
        eprintln!("error: {err}");
        std::process::exit(1);
    });
    println!(
        "Modelo: {} vértices, {} triángulos",
        obj.vertices.len(),
        obj.faces.len()
    );
    obj
}

/// Archivo OBJ → vértices + índices → triángulos → líneas → píxeles.
fn rasterize_model(obj: &Obj, view: View) -> Framebuffer {
    let transform = ScreenTransform::fit(&obj.vertices, view, WIDTH, HEIGHT);
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(BACKGROUND);
    framebuffer.clear();
    framebuffer.set_current_color(MODEL_COLOR);
    render(&mut framebuffer, obj, &transform);
    framebuffer
}

/// La vista frontal es `screenshot.png`; las demás `screenshot_<vista>.png`.
fn screenshot_path(view: View) -> PathBuf {
    let name = match view {
        View::Front => "screenshot.png".to_string(),
        other => format!("screenshot_{}.png", other.name()),
    };
    Path::new(EVIDENCE_DIR).join(name)
}

/// Renderiza todas las vistas sin abrir ventana y las guarda en evidence/.
fn screenshot() {
    let obj = load_model();
    for view in View::ALL {
        let path = screenshot_path(view);
        match rasterize_model(&obj, view).save_png(&path) {
            Ok(()) => println!("Captura {} guardada en {}", view.name(), path.display()),
            Err(err) => {
                eprintln!("error: {err}");
                std::process::exit(1);
            }
        }
    }
}

/// Muestra el framebuffer en una ventana; raylib solo sube la textura.
fn run_window() {
    let framebuffer = rasterize_model(&load_model(), View::Front);

    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("OBJ Wireframe Renderer - Untitled.obj (ESC para salir)")
        .build();
    rl.set_target_fps(60);
    rl.set_exit_key(Some(KeyboardKey::KEY_ESCAPE));

    let texture = rl
        .load_texture_from_image(&thread, &framebuffer.to_image())
        .expect("no se pudo crear la textura del framebuffer");

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }
}
