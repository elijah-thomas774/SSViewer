#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use eframe::{egui, egui_glow, glow};
use egui::mutex::Mutex;
use egui::panel::Side;
use egui::{Color32, Id, Response};
use ss_viewer::plc::{EntryType, ENTRY_FILTER};
use ss_viewer::scene::Scene;
// use stage_model::Stage;

use core::f32;
use std::fs;
use std::ops::RangeInclusive;
use std::sync::Arc;

mod file_formats;
mod gfx;
mod ss_viewer;

use gfx::shader::{Shader, ShaderUniformTypes};
use gfx::Model;

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
    model: Vec<Arc<Mutex<Scene>>>,
    selected_scene: Option<usize>,
    wireframe: bool,
    show_normals: bool,
    shader: Shader,
    nrm_shader: Shader,
    black_shader: Shader,
    cam_speed: f32,
    property_entry: usize,
    range_selection: u32,
    bg_color: Color32,
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

        let mut scene_map: Vec<Scene> = Vec::new();

        let stages =
            glob::glob(format!("{COLLISION_SRC_DIR}/*").as_str()).expect("Invalid Glob pattern");
        for stage_path in stages {
            let stage_path = stage_path.expect("Glob Error Encountered");
            scene_map.push(Scene::from_dir(stage_path).expect("Could not Read Stage"));
        }

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                                    Return                                             //
        ///////////////////////////////////////////////////////////////////////////////////////////

        Self {
            model: scene_map
                .into_iter()
                .map(|scene| Arc::new(Mutex::new(scene)))
                .collect(),
            selected_scene: None,
            wireframe: false,
            show_normals: false,
            shader,
            nrm_shader,
            black_shader,
            cam_speed: 30f32,
            property_entry: 0,
            range_selection: 0,
            bg_color: Color32::from_rgb(10, 10, 10),
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
            egui::ComboBox::from_label("Property Filter")
                .selected_text(format!(
                    "{}",
                    match &ENTRY_FILTER[self.property_entry] {
                        EntryType::Norm => format!("Normals"),
                        EntryType::Range(val) | EntryType::Single(val) => {
                            format!("Code {}: 0x{:08X}", val.code_idx, val.mask << val.shift)
                        }
                    }
                ))
                .show_ui(ui, |ui| {
                    for i in 0..ENTRY_FILTER.len() {
                        let text = match &ENTRY_FILTER[i] {
                            EntryType::Norm => format!("Normals"),
                            EntryType::Range(val) | EntryType::Single(val) => {
                                format!("Code {}: 0x{:08X}", val.code_idx, val.mask << val.shift)
                            }
                        };
                        if ui
                            .selectable_value(&mut self.property_entry, i, text)
                            .changed()
                        {
                            self.model.iter().for_each(|scene| {
                                let mut scene = scene.lock();
                                scene.update_scene_property_filter(i, self.range_selection);
                                scene.update_gl(frame.gl().unwrap());
                            });
                        }
                    }
                });
            match &ENTRY_FILTER[self.property_entry] {
                EntryType::Range(val) => {
                    if ui
                        .add(
                            egui::Slider::new(&mut self.range_selection, 0..=val.mask)
                                .clamp_to_range(true)
                                .hexadecimal(2, false, true),
                        )
                        .changed()
                    {
                        self.model.iter().for_each(|scene| {
                            let mut scene = scene.lock();
                            scene.update_scene_property_filter(
                                self.property_entry,
                                self.range_selection,
                            );
                        });

                        if let Some(scene_index) = self.selected_scene {
                            self.model[scene_index]
                                .lock()
                                .update_gl(frame.gl().unwrap());
                        }
                    }
                }
                _ => {}
            };
            ui.horizontal(|ui| {
                ui.color_edit_button_srgba(&mut self.bg_color);
                ui.label("BG Color");
            });

            ui.add(egui::Separator::default());

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.model.iter().enumerate().for_each(|(i, scene)| {
                        let mut scene = scene.lock();
                        if ui
                            .add(egui::SelectableLabel::new(
                                self.selected_scene == Some(i),
                                scene.get_root_name(),
                            ))
                            .clicked()
                        {
                            // No Need to change if it is the same
                            if Some(i) != self.selected_scene {
                                // Add the new gl
                                scene.setup_gl(frame.gl().unwrap());

                                // Remove the old gl
                                if let Some(old_scene) = self.selected_scene {
                                    self.model[old_scene].lock().destroy_gl(frame.gl().unwrap());
                                }

                                // Update Selection
                                self.selected_scene = Some(i);
                            }
                        }
                        if Some(i) == self.selected_scene {
                            scene.visibility_ui(ui);
                        }
                    });
                });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style())
                .fill(self.bg_color)
                .show(ui, |ui| {
                    self.custom_painting(ui, ctx);
                });
        });
        ctx.request_repaint();
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.shader.destroy(gl);
            self.nrm_shader.destroy(gl);
            self.model
                .iter_mut()
                .for_each(|stage| stage.lock().destroy_gl(gl));
        }
    }
}

impl MyApp {
    fn handle_input(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, response: &Response) {
        let _ = ui;
        if self.selected_scene.is_none() {
            return;
        }
        let selected_scene = self.selected_scene.unwrap();
        let scene = &mut self.model[selected_scene].lock();

        let cam = &mut scene.camera;

        ctx.input(|i| {
            let amount = (self.cam_speed * i.predicted_dt as f32 * 2.0f32) as f32;
            // let mut update_model = false;

            if i.key_down(egui::Key::W) {
                cam.move_forward(amount);
            }
            if i.key_down(egui::Key::S) {
                cam.move_backward(amount);
            }
            if i.key_down(egui::Key::A) {
                cam.move_left(amount);
            }
            if i.key_down(egui::Key::D) {
                cam.move_right(amount);
            }
            if i.key_down(egui::Key::Space) {
                cam.move_up(amount);
            }
            if i.modifiers.shift {
                cam.move_down(amount);
            }

            if i.key_down(egui::Key::Equals) {
                // self.scale(0.01);
            }
            if i.key_down(egui::Key::Minus) {
                // self.scale(-0.01);
            }
        });

        cam.move_yaw(response.drag_motion().x * 0.1);
        cam.move_pitch(-response.drag_motion().y * 0.1);
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // let size = ctx.input(|i| i.viewport().inner_rect.unwrap().size());
        if self.selected_scene.is_none() {
            return;
        }
        let size = ui.available_size();

        let (rect, response) = ui.allocate_at_least(size, egui::Sense::drag());

        let proj =
            glam::Mat4::perspective_rh_gl(70_f32.to_radians(), size.x / size.y, 1f32, 100000f32);
        // Handle Input related things
        self.handle_input(ui, ctx, &response);

        // Clone to Give to callback
        let scene = self.model[self.selected_scene.unwrap()].clone();
        let shader = self.shader.clone();
        let norm_shader = self.nrm_shader.clone();
        let black_shader = self.black_shader.clone();
        let wire_frame = self.wireframe;
        let show_normals = self.show_normals;
        let bg_color = self.bg_color;

        // Create Callback
        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                let scene = &mut scene.lock();
                let gl = painter.gl();
                unsafe {
                    use glow::HasContext as _;
                    gl.enable(glow::DEPTH_TEST);
                    if wire_frame {
                        gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                    } else {
                        gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
                    }
                    gl.clear_color(
                        bg_color.r() as f32 / u8::MAX as f32,
                        bg_color.g() as f32 / u8::MAX as f32,
                        bg_color.b() as f32 / u8::MAX as f32,
                        bg_color.a() as f32 / u8::MAX as f32,
                    );
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                }

                shader.use_program(gl);
                shader.set_uniform(gl, "proj", ShaderUniformTypes::Mat4(&proj));
                scene.draw(gl, &shader);

                // The Following code is used to render some normals for debugging
                if show_normals {
                    norm_shader.use_program(gl);
                    norm_shader.set_uniform(gl, "proj", ShaderUniformTypes::Mat4(&proj));
                    scene.draw(gl, &norm_shader);
                }

                unsafe {
                    use glow::HasContext as _;
                    // gl.clear(glow::DEPTH_BUFFER_BIT);
                    gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                }
                black_shader.use_program(gl);
                black_shader.set_uniform(gl, "proj", ShaderUniformTypes::Mat4(&proj));
                scene.draw(gl, &black_shader);

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
