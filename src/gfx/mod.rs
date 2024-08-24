pub mod camera;
pub mod mesh;
pub mod shader;
pub mod vertex;

pub use shader::Shader;
pub use vertex::Vertex;

use eframe::glow;

pub trait Model {
    fn setup_gl(&mut self, gl: &glow::Context);
    fn destroy_gl(&mut self, gl: &glow::Context);
    fn update_gl(&mut self, gl: &glow::Context);

    fn draw(&mut self, gl: &glow::Context, shader: &Shader);
}
