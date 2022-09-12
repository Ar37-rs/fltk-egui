use egui_backend::{
    egui,
    egui_glow::glow,
    fltk::{prelude::*, *},
    GlSurface,
    Api
};

use egui_demo_lib::DemoWindows;
use fltk::app::App;
use fltk_egui as egui_backend;
use std::rc::Rc;
use std::{cell::RefCell, time::Instant};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() {
    let fltk_app = app::App::default();
    let mut win = window::GlWindow::new(
        100,
        100,
        SCREEN_WIDTH as _,
        SCREEN_HEIGHT as _,
        Some("Demo window"),
    )
    .center_screen();
    win.set_mode(enums::Mode::MultiSample | enums::Mode::Alpha);
    win.end();
    win.make_resizable(true);
    win.show();
    win.make_current();

    let demo = egui_demo_lib::DemoWindows::default();
    run_egui(fltk_app, win, demo);
}

fn run_egui(fltk_app: App, mut win: window::GlWindow, mut demo: DemoWindows) {
    let (painter, egui_state) = egui_backend::with_fltk(&mut win, Api::OPENGL, true);
    let state = Rc::new(RefCell::new(egui_state));
    let painter = Rc::new(RefCell::new(painter));

    win.handle({
        let state = state.clone();
        move |win, ev| match ev {
            enums::Event::Push
            | enums::Event::Released
            | enums::Event::KeyDown
            | enums::Event::KeyUp
            | enums::Event::MouseWheel
            | enums::Event::Resize
            | enums::Event::Move
            | enums::Event::Drag
            | enums::Event::Activate => {
                // Using "if let ..." for safety.
                if let Ok(mut state) = state.try_borrow_mut() {
                    state.fuse_input(win, ev);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    });

    let egui_ctx = egui::Context::default();
    let start_time = Instant::now();
    win.draw({
        let painter = painter.clone();
        move |win| {
            let mut state = state.borrow_mut();
            let mut painter = painter.borrow_mut();
            
            // Clear the screen to dark red
            let gl = painter.gl().as_ref();
            draw_background(gl);
            state.input.time = Some(start_time.elapsed().as_secs_f64());
            let egui_output = egui_ctx.run(state.take_input(), |ctx| {
                demo.ui(ctx);
            });

            if egui_output.repaint_after.is_zero() || state.window_resized() {
                //Draw egui texture
                state.fuse_output(win, egui_output.platform_output);
                let meshes = egui_ctx.tessellate(egui_output.shapes);
                painter.paint_and_update_textures(
                    state.canvas_size,
                    state.pixels_per_point(),
                    &meshes,
                    &egui_output.textures_delta,
                );
                state.surface.swap_buffers(&state.gl_context).unwrap();
                app::awake();
            }
        }
    });

    let mut count = 0;
    while fltk_app.wait() {
        println!("flushing windows... {} times", count);
        win.flush();
        count += 1;
    }

    painter.borrow_mut().destroy();
}

fn draw_background<GL: glow::HasContext>(gl: &GL) {
    unsafe {
        gl.clear_color(0.6, 0.3, 0.3, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        gl.clear(glow::DEPTH_BUFFER_BIT);
    }
}
