use core::fmt;
use std::{error::Error, path::PathBuf};

use eframe::glow;
use glam::{Mat4, Vec3Swizzles};

use crate::gfx::{camera::Camera, Model, Shader};

use super::{DZBModel, KCLModel};

#[derive(Debug, Clone, Default)]
struct SceneNode {
    name: String,             // Name of the node (ex: "Room #")
    children: Vec<SceneNode>, // Can contain more children
    render: bool,
    kcl_model_idx: Vec<usize>, // Into kcl_models
    dzb_model_idx: Vec<usize>, // Into dzb_models
}

impl SceneNode {
    fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub camera: Camera,
    pub model_mat: Mat4,

    kcl_models: Vec<KCLModel>,
    dzb_models: Vec<DZBModel>,

    root_node: SceneNode,
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                     Errors Arrising when building a scene                                         //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum SceneError {
    InvalidRoot(PathBuf),
}
impl fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRoot(path) => write!(
                f,
                "Scene Root was not given a directory: {}",
                path.display()
            ),
        }
    }
}
impl Error for SceneError {}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                     Build The Scene Based on the directory                                        //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl Scene {
    // Builds a scene from a dir. private function
    fn build_scene(&mut self, dir: PathBuf) -> Option<SceneNode> {
        // The node name is derived from the name of the last folder/file in `dir`
        let node_name = dir
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        let mut node = SceneNode::default().with_name(node_name);

        // Compile List of Valid Files [Recursive]
        let mut kcl_files = Vec::new();
        let mut plc_files = Vec::new();
        let mut dzb_files = Vec::new();

        if let Ok(dirs) = dir.read_dir() {
            for dir in dirs {
                let path = dir.expect("Invalid DirEntry").path();
                if path.is_dir() {
                    if let Some(new_node) = self.build_scene(path) {
                        node.children.push(new_node);
                    }
                } else {
                    if let Some(extension) = path.extension() {
                        match extension.to_str().unwrap() {
                            "kcl" => kcl_files.push(path),
                            "dzb" => dzb_files.push(path),
                            "plc" => plc_files.push(path),
                            _ => {}
                        }
                    }
                }
            }
        }

        // Parse Files
        for kcl_path in kcl_files {
            // find plc
            let plc_path = plc_files
                .iter()
                .find(|&a| a.file_stem().unwrap() == kcl_path.file_stem().unwrap());
            if plc_path.is_none() {
                println!(
                    "Not displaying {}. Did not find a matching property (.plc) file",
                    kcl_path.display()
                );
                continue;
            }

            match KCLModel::from_file(kcl_path, plc_path.unwrap().into()) {
                Ok(kcl_model) => {
                    node.kcl_model_idx.push(self.kcl_models.len());
                    self.kcl_models.push(kcl_model);
                }
                Err(e) => {
                    println!("Unable to build KCLModel: {}", e);
                }
            }
        }

        for dzb_file in dzb_files {
            // find plc
            let plc_path = plc_files
                .iter()
                .find(|&a| a.file_stem().unwrap() == dzb_file.file_stem().unwrap());
            if plc_path.is_none() {
                println!(
                    "Not displaying {}. Did not find a matching property (.plc) file",
                    dzb_file.display()
                );
                continue;
            }

            match DZBModel::from_file(dzb_file, plc_path.unwrap().into()) {
                Ok(dzb_model) => {
                    node.dzb_model_idx.push(self.dzb_models.len());
                    self.dzb_models.push(dzb_model);
                }
                Err(e) => {
                    println!("Unable to build DZBModel: {}", e);
                }
            }
        }

        // Rendering the node is based off of:
        //  1. Containing room models -> Always Render
        //  2. Containing Children -> Always Render
        //  3. Containing ONLY Dzb -> Disable node, enable dzbs
        //  4. Constaining a mix with/DZB -> Enable the rest, but disable dzb

        // 5. If containing no children or models, dont include the node (None)

        let mut render_node = true;
        if node.kcl_model_idx.len() != 0 || node.children.len() != 0 {
            node.dzb_model_idx.iter().for_each(|&index| {
                self.dzb_models.get_mut(index).unwrap().render = false;
            });
        } else if node.dzb_model_idx.len() != 0 {
            render_node = false;
            node.dzb_model_idx.iter().for_each(|&index| {
                self.dzb_models.get_mut(index).unwrap().render = true;
            });
        } else {
            return None;
        }
        node.render = render_node;

        Some(node)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                            Outer Scene Interfacing                                                //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl Scene {
    fn new() -> Self {
        Self {
            camera: Camera::new(),
            kcl_models: Vec::new(),
            dzb_models: Vec::new(),
            model_mat: Mat4::IDENTITY,
            root_node: SceneNode::default(),
        }
    }

    pub fn from_dir(root_dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        // First check to see if the directory is valid for a scene
        if !root_dir.is_dir() {
            return Err(SceneError::InvalidRoot(root_dir).into());
        }

        // The following function loops through all directories and returns the graph of nodes
        let mut scene: Scene = Self::new();

        scene.root_node = scene.build_scene(root_dir).expect("Could not build Scene");

        Ok(scene)
    }

    pub fn update_scene_property_filter(&mut self, property_entry: usize, range_selection: u32) {
        self.kcl_models.iter_mut().for_each(|model| {
            model.properties.iter().enumerate().for_each(|(i, prop)| {
                let clr = prop
                    .get_color(property_entry, range_selection)
                    .unwrap_or_else(|| {
                        let v1 = model.verts[i * 3 + 0].pos;
                        let v2 = model.verts[i * 3 + 1].pos;
                        let v3 = model.verts[i * 3 + 2].pos;
                        (v2 - v1)
                            .cross(v3 - v1)
                            .normalize()
                            .abs()
                            .xyzx()
                            .with_w(1.0)
                    });
                model.verts[i * 3 + 0].clr = clr;
                model.verts[i * 3 + 1].clr = clr;
                model.verts[i * 3 + 2].clr = clr;
            });
        });
        self.dzb_models.iter_mut().for_each(|model| {
            model.properties.iter().enumerate().for_each(|(i, prop)| {
                let clr = prop
                    .get_color(property_entry, range_selection)
                    .unwrap_or_else(|| {
                        let v1 = model.verts[i * 3 + 0].pos;
                        let v2 = model.verts[i * 3 + 1].pos;
                        let v3 = model.verts[i * 3 + 2].pos;
                        (v2 - v1)
                            .cross(v3 - v1)
                            .normalize()
                            .abs()
                            .xyzx()
                            .with_w(1.0)
                    });
                model.verts[i * 3 + 0].clr = clr;
                model.verts[i * 3 + 1].clr = clr;
                model.verts[i * 3 + 2].clr = clr;
            });
        });
    }

    pub fn get_root_name(&self) -> String {
        self.root_node.name.clone()
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                                Scene Rendering                                                    //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl SceneNode {
    fn draw(
        &self,
        gl: &glow::Context,
        shader: &Shader,

        kcl_models: &mut Vec<KCLModel>,
        dzb_models: &mut Vec<DZBModel>,
    ) {
        if self.render {
            self.kcl_model_idx.iter().for_each(|&index| {
                kcl_models.get_mut(index).unwrap().draw(gl, shader);
            });
            self.dzb_model_idx.iter().for_each(|&index| {
                dzb_models.get_mut(index).unwrap().draw(gl, shader);
            });
            self.children.iter().for_each(|node| {
                node.draw(gl, shader, kcl_models, dzb_models);
            });
        }
    }
}

impl Model for Scene {
    fn setup_gl(&mut self, gl: &glow::Context) {
        self.kcl_models
            .iter_mut()
            .for_each(|model| model.setup_gl(gl));
        self.dzb_models
            .iter_mut()
            .for_each(|model| model.setup_gl(gl));
    }

    fn destroy_gl(&mut self, gl: &glow::Context) {
        self.kcl_models
            .iter_mut()
            .for_each(|model| model.destroy_gl(gl));
        self.dzb_models
            .iter_mut()
            .for_each(|model| model.destroy_gl(gl));
    }

    fn update_gl(&mut self, gl: &glow::Context) {
        self.kcl_models
            .iter_mut()
            .for_each(|model| model.update_gl(gl));
        self.dzb_models
            .iter_mut()
            .for_each(|model| model.update_gl(gl));
    }

    fn draw(&mut self, gl: &glow::Context, shader: &crate::gfx::Shader) {
        use crate::gfx::shader::ShaderUniformTypes;
        shader.use_program(gl);
        shader.set_uniform(gl, "view", ShaderUniformTypes::Mat4(&self.camera.get_mtx()));
        shader.set_uniform(gl, "model", ShaderUniformTypes::Mat4(&self.model_mat));

        self.root_node
            .draw(gl, shader, &mut self.kcl_models, &mut self.dzb_models);
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                               egui Interface                                                      //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl SceneNode {
    fn visibility_ui(
        &mut self,
        kcl_models: &mut Vec<KCLModel>,
        dzb_models: &mut Vec<DZBModel>,
        ui: &mut egui::Ui,
        show_name: bool,
    ) {
        if show_name {
            ui.checkbox(&mut self.render, self.name.clone());
        }
        if self.render {
            ui.indent(self.name.clone(), |ui| {
                self.kcl_model_idx.iter().for_each(|&model| {
                    let model = &mut kcl_models[model];
                    ui.checkbox(&mut model.render, model.name.clone());
                });
                self.dzb_model_idx.iter().for_each(|&model| {
                    let model = &mut dzb_models[model];
                    ui.checkbox(&mut model.render, model.name.clone());
                });

                self.children.iter_mut().for_each(|node| {
                    node.visibility_ui(kcl_models, dzb_models, ui, true);
                })
            });
        }
    }
}

impl Scene {
    pub fn visibility_ui(&mut self, ui: &mut egui::Ui) {
        self.root_node
            .visibility_ui(&mut self.kcl_models, &mut self.dzb_models, ui, false);
    }
}
