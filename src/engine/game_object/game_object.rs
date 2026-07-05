use std::collections::HashSet;

use crate::engine::gl_types::vectors::Vec3;

use super::{ComponentID, ObjectID};

pub struct Transform<'a> {
    pub(in crate::engine) obj: &'a mut GameObject
}

impl Transform<'_> {
    pub fn position(&self) -> &Vec3 {
        &self.obj.position
    }
    
    pub fn position_mut(&mut self) -> &mut Vec3 {
        &mut self.obj.position
    }
    
    pub fn rotation(&self) -> &Vec3 {
        &self.obj.rotation
    }
    
    pub fn rotation_mut(&mut self) -> &mut Vec3 {
        &mut self.obj.rotation
    }
    
    pub fn scale(&self) -> &Vec3 {
        &self.obj.scale
    }
    
    pub fn scale_mut(&mut self) -> &mut Vec3 {
        &mut self.obj.scale
    }
}

#[derive(Clone)]
pub(in crate::engine) struct GameObject {
    pub name: String,
    pub parent: ObjectID,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub components: Vec<ComponentID>,
    pub children: HashSet<ObjectID>
}