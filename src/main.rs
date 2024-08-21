#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use eframe::{egui, egui_glow, glow};
use egui::mutex::Mutex;
use egui::panel::Side;
use egui::{Id, Response};
use stage_model::Stage;

use core::f32;
use std::fs;
use std::ops::RangeInclusive;
use std::sync::Arc;

mod collision;
mod gfx;
mod ss_plc;
mod stage_model;

use gfx::shader::Shader;

const WIDTH: f32 = 1600f32;
const HEIGHT: f32 = 900f32;

const COLLISION_SRC_DIR: &str = "Collision Files";

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WIDTH, HEIGHT]),
        multisampling: 2,
        depth_buffer: 24,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Skyward Sword Collision Viewer",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
struct MyApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    model: Vec<Arc<Mutex<Stage>>>,
    selected_stage: Option<usize>,
    wireframe: bool,
    show_normals: bool,
    shader: Shader,
    nrm_shader: Shader,
    black_shader: Shader,
    cam_speed: f32,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");
        ///////////////////////////////////////////////////////////////////////////////////////////
        //                                    Build Shaders                                      //
        ///////////////////////////////////////////////////////////////////////////////////////////

        let vtx_shader = fs::read_to_string("Shaders/default.vs").expect("Could not find shader");
        let frag_shader = fs::read_to_string("Shaders/default.fs").expect("Could not find shader");
        let shader = Shader::from_src(gl, vtx_shader.as_str(), frag_shader.as_str(), None);

        let vtx_shader = fs::read_to_string("Shaders/normals.vs").expect("Could not find shader");
        let frag_shader = fs::read_to_string("Shaders/normals.fs").expect("Could not find shader");
        let geom_shader = fs::read_to_string("Shaders/normals.gs").expect("Could not find shader");
        let nrm_shader = Shader::from_src(
            gl,
            vtx_shader.as_str(),
            frag_shader.as_str(),
            Some(geom_shader.as_str()),
        );

        let vtx_shader = fs::read_to_string("Shaders/default.vs").expect("Could not find shader");
        let frag_shader = fs::read_to_string("Shaders/black.fs").expect("Could not find shader");
        let black_shader = Shader::from_src(gl, vtx_shader.as_str(), frag_shader.as_str(), None);

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                                    Load Stages                                        //
        ///////////////////////////////////////////////////////////////////////////////////////////

        let mut stage_map: Vec<Stage> = Vec::new();

        let stages =
            glob::glob(format!("{COLLISION_SRC_DIR}/*").as_str()).expect("Invalid Glob pattern");
        for stage_path in stages {
            let stage_path = stage_path.expect("Glob Error Encountered");
            stage_map.push(Stage::from_dir(gl, stage_path).expect("Could not Read Stage"));
        }

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                                    Return                                             //
        ///////////////////////////////////////////////////////////////////////////////////////////

        Self {
            model: stage_map
                .into_iter()
                .map(|stage| Arc::new(Mutex::new(stage)))
                .collect(),
            selected_stage: None,
            wireframe: false,
            show_normals: false,
            shader,
            nrm_shader,
            black_shader,
            cam_speed: 30f32,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::new(Side::Left, Id::new("Control Panel")).show(ctx, |ui| {
            ui.add(egui::Checkbox::new(&mut self.wireframe, "Wireframe"));
            ui.add(egui::Slider::new(
                &mut self.cam_speed,
                RangeInclusive::new(0.0, 1000.0),
            ));
            ui.add(egui::Separator::default());

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.model.iter().enumerate().for_each(|(i, stage)| {
                        let mut stage = stage.lock();
                        ui.selectable_value(&mut self.selected_stage, Some(i), stage.name.clone());
                        if let Some(select) = self.selected_stage {
                            if i == select {
                                stage.collision_selection(ui, frame);
                            }
                        }
                    });
                });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui, ctx);
            });
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.shader.destroy(gl);
            self.nrm_shader.destroy(gl);
            self.model
                .iter_mut()
                .for_each(|stage| stage.lock().destroy(gl));
        }
    }
}

impl MyApp {
    fn handle_input(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, response: &Response) {
        let _ = ui;
        if let Some(selected_stage) = self.selected_stage {
            self.model[selected_stage]
                .lock()
                .handle_input(ui, ctx, response, self.cam_speed);
        }
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // let size = ctx.input(|i| i.viewport().inner_rect.unwrap().size());
        if self.selected_stage == None {
            return;
        }
        let size = ui.available_size();

        let (rect, response) = ui.allocate_at_least(size, egui::Sense::drag());

        let proj =
            glam::Mat4::perspective_rh_gl(70_f32.to_radians(), size.x / size.y, 1f32, 100000f32);
        // Handle Input related things
        self.handle_input(ui, ctx, &response);

        // Clone to Give to callback
        let stage = self.model[self.selected_stage.unwrap()].clone();
        let shader = self.shader.clone();
        let norm_shader = self.nrm_shader.clone();
        let black_shader = self.black_shader.clone();
        let wire_frame = self.wireframe;
        let show_normals = self.show_normals;

        // Create Callback
        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                let stage = stage.lock();
                let gl = painter.gl();
                unsafe {
                    use glow::HasContext as _;
                    gl.enable(glow::DEPTH_TEST);
                    if wire_frame {
                        gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                    } else {
                        gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
                    }
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                }

                let view = stage.get_view();

                shader.use_program(gl);
                stage.draw(gl, &shader, &view, &proj);

                // The Following code is used to render some normals for debugging
                if show_normals {
                    norm_shader.use_program(gl);
                    stage.draw(gl, &norm_shader, &view, &proj);
                }

                unsafe {
                    use glow::HasContext as _;
                    // gl.clear(glow::DEPTH_BUFFER_BIT);
                    gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                }
                black_shader.use_program(gl);
                stage.draw(gl, &black_shader, &view, &proj);

                // Reset back to the normal setting
                unsafe {
                    use glow::HasContext as _;
                    gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
                }
            })),
        };
        ui.painter().add(callback);
    }
}
