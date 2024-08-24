use std::{
    error::Error, fmt, fs, io::Cursor, mem::offset_of, path::PathBuf, ptr::slice_from_raw_parts,
};

use crate::{
    file_formats::{PLCEntry, DZB, PLC},
    gfx::{Model, Shader, Vertex},
};
use eframe::glow;
use glam::{Vec3, Vec3Swizzles};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DZBModel {
    // Root File
    pub name: String,
    file: DZB,

    // Properties of each vertex -> prop_i -> (vtx_3i, vtx_3i+1, vtx_3i+2)
    pub properties: Vec<PLCEntry>,

    // Rendering Information
    pub render: bool,
    pub verts: Vec<Vertex>,
    vao: Option<glow::VertexArray>,
    vbo: Option<glow::Buffer>,
}

#[derive(Debug, Clone)]
enum DZBError {
    InvalidPLC,
}
impl fmt::Display for DZBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPLC => write!(f, "DZBError: DZB and PLC files do not match"),
        }
    }
}
impl std::error::Error for DZBError {}

impl DZBModel {
    pub fn from_file(dzb_path: PathBuf, plc_path: PathBuf) -> Result<Self, Box<dyn Error>> {
        // The Game always has the plc and the collision files share the same stem
        if dzb_path.file_stem().unwrap() != plc_path.file_stem().unwrap() {
            return Err(DZBError::InvalidPLC.into());
        }

        // The name for the model for display purposes
        let name = dzb_path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .expect("Could not make String from OsString");
        println!("{}: Creating DZB Model", dzb_path.display());

        // Read The PLC and the DZB File
        let dzb = DZB::from_file(&mut Cursor::new(&fs::read(dzb_path)?))?;
        let plc = PLC::from_file(&mut Cursor::new(&fs::read(plc_path)?))?;

        // The Important things for the model are vertices to draw
        // The DZB Stores Triangles directly, so no need to extract them manually
        let tris = &dzb.tris;

        // Since the the plc file is index'd into and we may want to edit a single triangle,
        //  each property needs to be duplicated. On rebuilding, it should perform de-duplication
        let mut vtx_array =
            Vec::<Vertex>::with_capacity(tris.len() * 3 /* Three Verts per Tri */);
        let mut prop_array =
            Vec::<PLCEntry>::with_capacity(tris.len() /* One Property per Tri */);
        tris.iter().for_each(|tri| {
            let verts: Vec<Vec3> = tri
                .vert_idx
                .iter()
                .map(|&idx| dzb.verts[idx as usize])
                .collect();

            let nrm = (verts[1] - verts[0]).cross(verts[2] - verts[0]).normalize();
            let clr = nrm.abs().xyzx().with_w(1.0);

            verts
                .iter()
                .for_each(|vtx| vtx_array.push(Vertex::new(vtx.clone(), nrm, clr)));
            prop_array.push(plc.entries[tri.prop_idx as usize].clone());
        });

        let model = Self {
            name,
            file: dzb,
            render: true,
            verts: vtx_array,
            properties: prop_array,
            vao: None,
            vbo: None,
        };

        Ok(model)
    }
}

impl Model for DZBModel {
    fn setup_gl(&mut self, gl: &glow::Context) {
        // Do not setup twice!
        if self.vao.is_some() || self.vbo.is_some() {
            return;
        }

        unsafe {
            use glow::HasContext as _;

            // Create Vertex Array and Vertex Buffer
            match gl.create_vertex_array() {
                Ok(vao) => self.vao = Some(vao),
                Err(e) => panic!("{}", e),
            };
            match gl.create_buffer() {
                Ok(vbo) => self.vbo = Some(vbo),
                Err(e) => panic!("{}", e),
            };

            gl.bind_vertex_array(self.vao);
            gl.bind_buffer(glow::ARRAY_BUFFER, self.vbo);

            let bind_data = slice_from_raw_parts(
                self.verts.as_ptr() as *const u8,
                self.verts.len() * size_of::<Vertex>(),
            )
            .as_ref()
            .unwrap();
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

            gl.bind_vertex_array(None)
        }
    }

    fn destroy_gl(&mut self, gl: &glow::Context) {
        unsafe {
            use glow::HasContext as _;

            if let (Some(vao), Some(vbo)) = (self.vao, self.vbo) {
                gl.delete_vertex_array(vao);
                gl.delete_buffer(vbo);
            }

            self.vao = None;
            self.vbo = None;
        }
    }

    fn update_gl(&mut self, gl: &glow::Context) {
        unsafe {
            use glow::HasContext as _;

            if self.vao.is_some() && self.vbo.is_some() {
                gl.bind_buffer(glow::ARRAY_BUFFER, self.vbo);
                let bind_data = slice_from_raw_parts::<u8>(
                    self.verts.as_ptr() as *const u8,
                    self.verts.len() * size_of::<Vertex>(),
                )
                .as_ref()
                .unwrap();

                gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, bind_data);
            }
        }
    }

    fn draw(&mut self, gl: &glow::Context, shader: &Shader) {
        // KCL Models are usually part of a stage. All uniforms should belong to the stage
        let _ = shader;

        if !self.render {
            return;
        }

        if self.vao.is_none() || self.vbo.is_none() {
            return;
        }

        unsafe {
            use glow::HasContext as _;

            gl.bind_vertex_array(self.vao);
            gl.draw_arrays(glow::TRIANGLES, 0, self.verts.len() as _);
        }
    }
}
