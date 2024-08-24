use eframe::glow;
use glam::{Mat4, Vec2, Vec3, Vec4};

#[allow(dead_code)]
pub enum ShaderUniformTypes<'a> {
    Mat4(&'a Mat4),
    Vec4(&'a Vec4),
    Vec3(&'a Vec3),
    Vec2(&'a Vec2),
    F32(&'a f32),
    U32(&'a u32),
    I32(&'a i32),
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Shader(glow::Program);

impl Shader {
    pub fn from_src(gl: &glow::Context, vtx: &str, frag: &str, geom: Option<&str>) -> Self {
        use glow::HasContext as _;

        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let mut shader_sources = Vec::new();
            shader_sources.push((glow::VERTEX_SHADER, vtx));
            shader_sources.push((glow::FRAGMENT_SHADER, frag));
            if let Some(geom) = geom {
                shader_sources.push((glow::GEOMETRY_SHADER, geom));
            }

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(shader, shader_source);
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();

            gl.link_program(program);
            assert!(
                gl.get_program_link_status(program),
                "{}",
                gl.get_program_info_log(program)
            );

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            Self(program)
        }
    }

    pub fn set_uniform(&self, gl: &glow::Context, name: &str, uniform: ShaderUniformTypes) {
        unsafe {
            use glow::HasContext as _;
            match uniform {
                ShaderUniformTypes::Mat4(uniform) => {
                    gl.uniform_matrix_4_f32_slice(
                        gl.get_uniform_location(self.0, name).as_ref(),
                        false,
                        &uniform.to_cols_array(),
                    );
                }
                ShaderUniformTypes::Vec4(uniform) => {
                    gl.uniform_4_f32_slice(
                        gl.get_uniform_location(self.0, name).as_ref(),
                        &uniform.to_array(),
                    );
                }
                ShaderUniformTypes::Vec3(uniform) => {
                    gl.uniform_3_f32_slice(
                        gl.get_uniform_location(self.0, name).as_ref(),
                        &uniform.to_array(),
                    );
                }
                ShaderUniformTypes::Vec2(uniform) => {
                    gl.uniform_2_f32_slice(
                        gl.get_uniform_location(self.0, name).as_ref(),
                        &uniform.to_array(),
                    );
                }
                ShaderUniformTypes::F32(uniform) => {
                    gl.uniform_1_f32(gl.get_uniform_location(self.0, name).as_ref(), *uniform);
                }
                ShaderUniformTypes::U32(uniform) => {
                    gl.uniform_1_u32(gl.get_uniform_location(self.0, name).as_ref(), *uniform);
                }
                ShaderUniformTypes::I32(uniform) => {
                    gl.uniform_1_i32(gl.get_uniform_location(self.0, name).as_ref(), *uniform);
                }
            }
        }
    }

    pub fn use_program(&self, gl: &glow::Context) {
        unsafe {
            use glow::HasContext as _;

            gl.use_program(Some(self.0));
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            use glow::HasContext as _;

            gl.delete_program(self.0);
        }
    }
}
