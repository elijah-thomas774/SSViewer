use glam::{self, Mat4, Vec3};

#[derive(Debug, Clone)]
pub struct Camera {
    pos: Vec3,
    front: Vec3,
    up: Vec3,
    pitch: f32,
    yaw: f32,

    mtx: Mat4,
    dirty: bool,
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                               Creation Functions                                                  //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
impl Camera {
    pub fn new() -> Self {
        Self {
            pos: Vec3::ZERO,
            front: Vec3::Z,
            up: Vec3::Y,
            pitch: 0.0f32,
            yaw: 0.0f32,

            mtx: Mat4::IDENTITY,
            dirty: true,
        }
    }

    pub fn with_pos(mut self, pos: Vec3) -> Self {
        self.pos = pos;
        self.dirty = true;
        self
    }

    pub fn with_front(mut self, front: Vec3) -> Self {
        self.front = front.normalize();
        self.dirty = true;
        self
    }

    pub fn with_up(mut self, up: Vec3) -> Self {
        self.up = up.normalize();
        self.dirty = true;
        self
    }

    pub fn with_yaw(mut self, yaw: f32) -> Self {
        self.yaw = yaw;
        self.dirty = true;
        self
    }

    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(-89.0, 89.0);
        self.dirty = true;
        self
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                              Usability Functions                                                  //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
impl Camera {
    pub fn translate(&mut self, translate: Vec3) {
        self.pos += translate;
        self.dirty = true;
    }

    pub fn move_forward(&mut self, amount: f32) {
        self.pos += amount * self.front;
        self.dirty = true;
    }

    pub fn move_backward(&mut self, amount: f32) {
        self.pos -= amount * self.front;
        self.dirty = true;
    }

    pub fn move_right(&mut self, amount: f32) {
        self.pos += self.front.cross(self.up).normalize() * amount;
        self.dirty = true;
    }

    pub fn move_left(&mut self, amount: f32) {
        self.pos -= self.front.cross(self.up).normalize() * amount;
        self.dirty = true;
    }

    pub fn move_up(&mut self, amount: f32) {
        self.pos.y += amount;
        self.dirty = true;
    }

    pub fn move_down(&mut self, amount: f32) {
        self.pos.y -= amount;
        self.dirty = true;
    }

    pub fn move_pitch(&mut self, amount: f32) {
        self.pitch = (self.pitch + amount).clamp(-89.0, 89.0);
        self.dirty = true;
    }

    pub fn move_yaw(&mut self, amount: f32) {
        self.yaw += amount;
        self.dirty = true;
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                              Get / Set Functions                                                  //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
impl Camera {
    pub fn get_mtx(&mut self) -> Mat4 {
        if self.dirty {
            self.calc_mtx();
        }

        self.mtx
    }

    pub fn get_pos(&self) -> Vec3 {
        self.pos
    }

    pub fn get_front(&self) -> Vec3 {
        self.front
    }

    pub fn get_up(&self) -> Vec3 {
        self.up
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        self.pos = pos;
        self.dirty = true;
    }

    pub fn set_front(&mut self, front: Vec3) {
        self.front = front.normalize();
        self.dirty = true;
    }

    pub fn set_up(&mut self, up: Vec3) {
        self.up = up.normalize();
        self.dirty = true;
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                               Internal Functions                                                  //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl Camera {
    fn calc_mtx(&mut self) {
        self.front = Vec3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        )
        .normalize();

        self.mtx = Mat4::look_at_rh(self.pos, self.pos + self.front, self.up);

        self.dirty = false;
    }
}
