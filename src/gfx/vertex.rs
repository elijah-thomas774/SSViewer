#[derive(Debug, Clone)]
#[repr(C)]
pub struct Vertex {
    pub pos: glam::Vec3,
    pub nrm: glam::Vec3,
    pub clr: glam::Vec4,
}

impl Vertex {
    pub fn new(pos: glam::Vec3, nrm: glam::Vec3, clr: glam::Vec4) -> Self {
        Self {
            pos,
            nrm: nrm.normalize(),
            clr,
        }
    }
}
