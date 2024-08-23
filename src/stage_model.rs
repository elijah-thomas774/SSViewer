use std::{
    fs, io::Cursor, iter::zip, mem::offset_of, path::PathBuf, ptr::slice_from_raw_parts,
    str::FromStr, usize,
};

use eframe::glow;
use egui::{self, Response};
use glam::{Mat4, Quat, Vec3, Vec3Swizzles, Vec4};

use crate::{
    collision::{
        dzb::DZB,
        kcl::KCL,
        plc::{PLCEntry, PLC},
    },
    gfx::{
        shader::{Shader, ShaderUniformTypes},
        Vertex,
    },
    stage_model,
};

impl KCL {
    pub fn get_verts(&self, gl: &glow::Context, plc: &PLC) -> (Vec<Vertex>, Vec<PLCEntry>) {
        let _ = gl;
        let mut verts = Vec::<Vertex>::new();
        let mut props = Vec::<PLCEntry>::new();

        for prism in &self.prisms {
            for vert in prism.vertices {
                verts.push(Vertex::new(
                    vert,
                    prism.face_normal,
                    prism.face_normal.abs().xyzx().with_w(1.0f32),
                ));
            }
            props.push(plc.entries[prism.attribute as usize].clone());
        }

        (verts, props)
    }
}

impl DZB {
    pub fn get_verts(&self, gl: &glow::Context, plc: &PLC) -> (Vec<Vertex>, Vec<PLCEntry>) {
        let _ = gl;
        let mut verts = Vec::<Vertex>::new();
        let mut props = Vec::<PLCEntry>::new();

        for tri in &self.tris {
            let v1: Vec3 = self.verts[tri.vert_idx[0] as usize];
            let v2: Vec3 = self.verts[tri.vert_idx[1] as usize];
            let v3: Vec3 = self.verts[tri.vert_idx[2] as usize];

            let nrm = (v2 - v1).cross(v3 - v1).normalize();
            verts.push(Vertex::new(v1, nrm, nrm.abs().xyzx().with_w(1.0f32)));
            verts.push(Vertex::new(v2, nrm, nrm.abs().xyzx().with_w(1.0f32)));
            verts.push(Vertex::new(v3, nrm, nrm.abs().xyzx().with_w(1.0f32)));
            props.push(plc.entries[tri.prop_idx as usize].clone());
        }

        (verts, props)
    }
}

#[derive(Debug, Clone)]
pub struct Stage {
    pub name: String,
    cam_pos: Vec3,
    cam_front: Vec3,
    cam_up: Vec3,
    pitch: f32,
    yaw: f32,
    pos: Vec3,
    rot: Vec3,
    scale: Vec3,
    model_mat: Mat4,
    rooms: Vec<Room>,
}

#[derive(Debug, Clone)]
struct Room {
    name: String,
    render: bool,
    models: Vec<CollisionModel>,
}

#[derive(Debug, Clone)]
struct CollisionModel {
    name: String,
    render: bool,
    // kcl: KCL,
    tri_properties: Vec<PLCEntry>,
    // dzb : Vec<DZB>, // NYI
    tris: Vec<Vertex>,

    vao: glow::VertexArray,
    vbo: glow::Buffer,
}

///////////////////////////////////////////////////////////////////////////////////////////////////
///                                    Derivations                                              ///
///////////////////////////////////////////////////////////////////////////////////////////////////

impl PartialEq for Stage {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialEq for Room {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialEq for CollisionModel {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
///                                    Building and Editing                                     ///
///////////////////////////////////////////////////////////////////////////////////////////////////

impl Stage {
    pub fn from_dir(gl: &glow::Context, dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // Stage is the Key Value
        let stage_str = String::from_str(
            dir.file_name()
                .expect("Should be valid file")
                .to_str()
                .unwrap(),
        )?;

        let mut stage = Stage {
            name: stage_str.clone(),
            cam_pos: Vec3::new(0f32, 0f32, -1000f32),
            cam_front: Vec3::new(0f32, 0f32, -1f32),
            cam_up: Vec3::new(0f32, 1f32, 0f32),
            pitch: 0f32,
            yaw: 0f32,
            pos: Vec3::ZERO,
            rot: Vec3::ZERO,
            scale: Vec3::splat(1.0),
            model_mat: Mat4::IDENTITY,
            rooms: Vec::new(),
        };

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                      Read Stage DZB and PLC entries                                   //
        ///////////////////////////////////////////////////////////////////////////////////////////

        let mut room = Room {
            name: String::from("Stage Additions"),
            render: true,
            models: Vec::new(),
        };

        let dzb_files_names = glob::glob(format!("{}/addon/*.dzb", dir.display()).as_str())
            .expect("Invalid Glob pattern");
        let plc_files_names = glob::glob(format!("{}/addon/*.plc", dir.display()).as_str())
            .expect("Invalid Glob pattern");
        // Make sure the entries match then parse/insert into the room
        for entry in zip(dzb_files_names, plc_files_names) {
            if let (Ok(dzb_path), Ok(plc_path)) = entry {
                // Check File names
                let dzb_file_name = dzb_path.file_stem().expect("Not a File");
                let plc_file_name = plc_path.file_stem().expect("Not a File");

                let dzb = DZB::from_file(&mut Cursor::new(&fs::read(dzb_path.clone())?))?;
                let plc = PLC::from_file(&mut Cursor::new(&fs::read(plc_path.clone())?))?;

                print!("{}: ", stage_str);
                let mut model = CollisionModel::from_dzb(
                    gl,
                    String::from_str(dzb_file_name.to_str().unwrap()).unwrap(),
                    &dzb,
                    &plc,
                );

                model.render = false;

                room.models.push(model);
            }
        }

        if room.models.len() != 0 {
            stage.rooms.push(room);
        }

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                             Read Stage Room entries                                   //
        ///////////////////////////////////////////////////////////////////////////////////////////

        let rooms = glob::glob(format!("{}/rooms/*", dir.display()).as_str())?;

        for room_path_str in rooms {
            let room_path_str = room_path_str?;

            let room_name =
                String::from_str(room_path_str.file_name().unwrap().to_str().unwrap()).unwrap();

            // Create Room
            let mut room = Room {
                name: room_name.clone(),
                render: true,
                models: Vec::new(),
            };

            // Grab related Files
            let kcl_files_names = glob::glob(format!("{}/*.kcl", room_path_str.display()).as_str())
                .expect("Failed Glob Pattern");

            let plc_files_names = glob::glob(format!("{}/*.plc", room_path_str.display()).as_str())
                .expect("Failed Glob Pattern");

            // Make sure the entries match then parse/insert into the room
            for entry in zip(kcl_files_names, plc_files_names) {
                if let (Ok(kcl_path), Ok(plc_path)) = entry {
                    // Check File names
                    let kcl_file_name = kcl_path.file_stem().expect("Not a File");
                    let plc_file_name = plc_path.file_stem().expect("Not a File");

                    let kcl = KCL::from_file(&mut Cursor::new(&fs::read(kcl_path.clone())?))?;
                    let plc = PLC::from_file(&mut Cursor::new(&fs::read(plc_path.clone())?))?;

                    print!("{} {}: ", stage_str, room_name);
                    let model = CollisionModel::from_kcl(
                        gl,
                        String::from_str(kcl_file_name.to_str().unwrap()).unwrap(),
                        &kcl,
                        &plc,
                    );

                    room.models.push(model);
                }
            }
            stage.rooms.push(room);
        }

        ///////////////////////////////////////////////////////////////////////////////////////////
        //                             Read Oarc entries                                         //
        ///////////////////////////////////////////////////////////////////////////////////////////

        if stage_str.contains("Oarc") {
            let object_dirs =
                glob::glob(format!("{}/*", dir.display()).as_str()).expect("Failed Glob Pattern");
            for obj_path in object_dirs {
                if let Ok(obj_path) = obj_path {
                    let obj_name =
                        String::from_str(obj_path.file_name().unwrap().to_str().unwrap()).unwrap();
                    let mut room = Room {
                        name: obj_name.clone(),
                        render: false,
                        models: Vec::new(),
                    };
                    let dzb_files_names =
                        glob::glob(format!("{}/*.dzb", obj_path.display()).as_str())
                            .expect("Invalid Glob pattern");
                    let plc_files_names =
                        glob::glob(format!("{}/*.plc", obj_path.display()).as_str())
                            .expect("Invalid Glob pattern");
                    // Make sure the entries match then parse/insert into the room
                    for entry in zip(dzb_files_names, plc_files_names) {
                        if let (Ok(dzb_path), Ok(plc_path)) = entry {
                            // Check File names
                            let dzb_file_name = dzb_path.file_stem().expect("Not a File");
                            let plc_file_name = plc_path.file_stem().expect("Not a File");

                            let dzb =
                                DZB::from_file(&mut Cursor::new(&fs::read(dzb_path.clone())?))?;
                            let plc =
                                PLC::from_file(&mut Cursor::new(&fs::read(plc_path.clone())?))?;

                            print!("{} {}: ", stage_str, obj_name);
                            let model = CollisionModel::from_dzb(
                                gl,
                                String::from_str(dzb_file_name.to_str().unwrap()).unwrap(),
                                &dzb,
                                &plc,
                            );

                            room.models.push(model);
                        }
                    }
                    stage.rooms.push(room);
                }
            }
        }

        stage.update();

        Ok(stage)
    }

    pub fn scale(&mut self, scale: f32) {
        self.scale += scale;
    }
    pub fn update(&mut self) {
        self.model_mat = Mat4::from_scale_rotation_translation(
            self.scale,
            Quat::from_rotation_x(self.rot.x)
                * Quat::from_rotation_y(self.rot.y)
                * Quat::from_rotation_x(self.rot.z),
            self.pos,
        );
    }

    pub fn get_view(&self) -> Mat4 {
        Mat4::look_at_rh(self.cam_pos, self.cam_pos + self.cam_front, self.cam_up)
    }
}

impl CollisionModel {
    fn from_kcl(gl: &glow::Context, name: String, kcl: &KCL, plc: &PLC) -> Self {
        let (tris, tri_properties) = kcl.get_verts(gl, &plc);

        println!("{}: Creating KCL Model", name.clone());

        unsafe {
            use glow::HasContext as _;

            // Create Vertex Array
            let vao = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            // Create Vertex Buffer
            let vbo = gl.create_buffer().expect("Cannot create vertex buffer");

            let bind_data = slice_from_raw_parts::<u8>(
                tris.as_ptr() as *const u8,
                tris.len() * size_of::<Vertex>(),
            )
            .as_ref()
            .unwrap();

            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bind_data, glow::STATIC_DRAW);

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, size_of::<Vertex>() as _, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                size_of::<Vertex>() as _,
                offset_of!(Vertex, nrm) as _,
            );
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(
                2,
                4,
                glow::FLOAT,
                false,
                size_of::<Vertex>() as _,
                offset_of!(Vertex, clr) as _,
            );

            gl.bind_vertex_array(None);
            Self {
                name,
                render: true,
                // kcl,
                tri_properties,
                tris,
                vao,
                vbo,
            }
        }
    }

    pub fn from_dzb(gl: &glow::Context, name: String, dzb: &DZB, plc: &PLC) -> Self {
        let (tris, tri_properties) = dzb.get_verts(gl, &plc);

        println!("{}: Creating DZB Model", name.clone());

        unsafe {
            use glow::HasContext as _;

            // Create Vertex Array
            let vao = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            // Create Vertex Buffer
            let vbo = gl.create_buffer().expect("Cannot create vertex buffer");

            let bind_data = slice_from_raw_parts::<u8>(
                tris.as_ptr() as *const u8,
                tris.len() * size_of::<Vertex>(),
            )
            .as_ref()
            .unwrap();

            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bind_data, glow::STATIC_DRAW);

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, size_of::<Vertex>() as _, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                size_of::<Vertex>() as _,
                offset_of!(Vertex, nrm) as _,
            );
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(
                2,
                4,
                glow::FLOAT,
                false,
                size_of::<Vertex>() as _,
                offset_of!(Vertex, clr) as _,
            );

            gl.bind_vertex_array(None);
            Self {
                name,
                render: true,
                // kcl,
                tri_properties,
                tris,
                vao,
                vbo,
            }
        }
    }

    pub fn update_tri(&mut self, gl: &glow::Context, property_entry: usize, range_selection: u32) {
        for tri in &mut self.tris {
            tri.clr = Vec4::splat(0.5f32);
        }
        unsafe {
            use glow::HasContext as _;

            self.tri_properties
                .iter()
                .enumerate()
                .for_each(|(i, prop)| {
                    let clr = prop.get_color(property_entry, range_selection);
                    match clr {
                        Some(clr) => {
                            self.tris[i * 3 + 0].clr = clr;
                            self.tris[i * 3 + 1].clr = clr;
                            self.tris[i * 3 + 2].clr = clr;
                        }
                        None => {
                            let nrm = self.tris[i * 3].nrm.abs().xyzx().with_w(1.0f32);
                            self.tris[i * 3 + 0].clr = nrm;
                            self.tris[i * 3 + 1].clr = nrm;
                            self.tris[i * 3 + 2].clr = nrm;
                        }
                    }
                });

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            let bind_data = slice_from_raw_parts::<u8>(
                self.tris.as_ptr() as *const u8,
                self.tris.len() * size_of::<Vertex>(),
            )
            .as_ref()
            .unwrap();

            gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, bind_data);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
///                                    Egui Render Stuff                                        ///
///////////////////////////////////////////////////////////////////////////////////////////////////

impl Stage {
    pub fn handle_input(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        response: &Response,
        speed: f32,
    ) {
        let _ = ui;
        ctx.input(|i| {
            let step = (speed * i.predicted_dt as f32 * 2.0f32) as f32;
            let mut update_model = false;

            if i.key_down(egui::Key::W) {
                self.cam_pos += step * self.cam_front;
            }
            if i.key_down(egui::Key::S) {
                self.cam_pos -= step * self.cam_front;
            }
            if i.key_down(egui::Key::A) {
                self.cam_pos -= self.cam_front.cross(self.cam_up).normalize() * step;
            }
            if i.key_down(egui::Key::D) {
                self.cam_pos += self.cam_front.cross(self.cam_up).normalize() * step;
            }
            if i.key_down(egui::Key::Space) {
                self.cam_pos += Vec3::new(0.0f32, step, 0f32);
            }
            if i.modifiers.shift {
                self.cam_pos -= Vec3::new(0.0f32, step, 0f32);
            }

            if i.key_down(egui::Key::Equals) {
                self.scale(0.01);
                update_model = true;
            }
            if i.key_down(egui::Key::Minus) {
                self.scale(-0.01);
                update_model = true;
            }

            if update_model {
                self.update();
            }
        });
        self.yaw += response.drag_motion().x * 0.1;
        self.pitch -= response.drag_motion().y * 0.1;
        self.pitch = self.pitch.clamp(-89.5f32, 89.5f32);
        self.cam_front = Vec3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        )
        .normalize();
    }

    pub fn destroy(&mut self, gl: &glow::Context) {
        for room in &mut self.rooms {
            room.destroy(gl);
        }
    }

    pub fn draw(&self, gl: &glow::Context, shader: &Shader, view: &Mat4, proj: &Mat4) {
        for room in &self.rooms {
            shader.set_uniform(gl, "model", ShaderUniformTypes::Mat4(&self.model_mat));
            shader.set_uniform(gl, "view", ShaderUniformTypes::Mat4(&view));
            shader.set_uniform(gl, "proj", ShaderUniformTypes::Mat4(&proj));
            room.draw(gl, shader);
        }
    }
}

impl Room {
    pub fn destroy(&mut self, gl: &glow::Context) {
        for model in &mut self.models {
            model.destroy(gl);
        }
    }

    pub fn draw(&self, gl: &glow::Context, shader: &Shader) {
        // If Render the Room in General
        if self.render {
            for model in &self.models {
                // If render that particular section in general
                if model.render {
                    model.draw(gl, shader);
                }
            }
        }
    }
}

impl CollisionModel {
    pub fn destroy(&mut self, gl: &glow::Context) {
        unsafe {
            use glow::HasContext as _;
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
        }
    }

    pub fn draw(&self, gl: &glow::Context, shader: &Shader) {
        unsafe {
            use glow::HasContext as _;

            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, self.tris.len() as _);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
///                                    Egui UI Stuffs                                           ///
///////////////////////////////////////////////////////////////////////////////////////////////////

impl Stage {
    pub fn collision_selection(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Show Rooms
        ui.indent(self.name.clone(), |ui| {
            for room in &mut self.rooms {
                ui.checkbox(&mut room.render, room.name.clone());
                if room.render {
                    ui.indent(self.name.clone(), |ui| {
                        for model in &mut room.models {
                            ui.checkbox(&mut model.render, format!("{}", model.name));
                        }
                    });
                }
            }
        });
    }

    pub fn update_tris(&mut self, gl: &glow::Context, property_entry: usize, range_selection: u32) {
        for room in &mut self.rooms {
            for model in &mut room.models {
                model.update_tri(gl, property_entry, range_selection);
            }
        }
    }
}
