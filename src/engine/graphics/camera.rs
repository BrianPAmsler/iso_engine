use crate::engine::gl_types::{clip_space::{ortho_aspect, perspective}, matrices::Mat4, transform::look_at, vectors::Vec3};

#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Orthographic {
        width: f32, 
        aspect: f32,
        z_near: f32,
        z_far: f32
    },
    Perspective {
        fovx: f32,
        aspect: f32,
        near: f32,
        far: f32
    }
}

pub struct Camera {
    projection_matrix: Option<Mat4>,
    view_matrix: Option<Mat4>,
    projection: Projection,
    position: Vec3,
    direction: Vec3,
    up: Vec3,
}

impl Camera {
    pub fn new(projection: Projection, position: Vec3, direction: Vec3, up: Vec3) -> Camera {
        
        Camera { projection, projection_matrix: None, view_matrix: None, position, direction, up }
    }

    pub fn projection(&self) -> &Projection {
        &self.projection
    }

    pub fn projection_mut(&mut self) -> &mut Projection {
        self.projection_matrix = None;
        &mut self.projection
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.view_matrix = None;
        self.position = position;
    }

    pub fn up(&self) -> Vec3 {
        self.up
    }

    pub fn set_up(&mut self, up: Vec3) {
        self.view_matrix = None;
        self.up = up;
    }

    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    pub fn set_direction(&mut self, direction: Vec3) {
        self.view_matrix = None;
        self.direction = direction;
    }

    pub fn view_matrix(&mut self) -> Mat4 {
        if self.view_matrix.is_none() {
            self.view_matrix = Some(look_at(self.position, self.position + self.direction, self.up));
        }

        #[allow(clippy::unwrap_used, reason="if statement ensures view_matrix is always Some")]
        self.view_matrix.unwrap()
    }

    pub fn projection_matrix(&mut self) -> Mat4 {
        if self.projection_matrix.is_none() {
            self.projection_matrix = match self.projection {
                Projection::Orthographic { width, aspect, z_near, z_far } => Some(ortho_aspect(width, aspect, z_near, z_far)),
                Projection::Perspective { fovx, aspect, near, far } => Some(perspective(fovx, aspect, near, far)),
            };
        }

        #[allow(clippy::unwrap_used, reason="if statement ensures view_matrix is always Some")]
        self.projection_matrix.unwrap()
    }

    pub fn update_aspect(&mut self, aspect: f32) {
        let projection_aspect = match &mut self.projection {
            Projection::Orthographic { aspect, .. } => aspect,
            Projection::Perspective {  aspect, .. } => aspect,
        };

        *projection_aspect = aspect;
        self.projection_matrix = None;
    }
}