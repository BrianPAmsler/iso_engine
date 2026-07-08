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
    pub components: Vec<ComponentID<()>>,
    pub children: HashSet<ObjectID>
}

pub(in crate::engine::game_object) mod serialize {

    use crate::engine::{game_object::{ObjectID, World}, gl_types::vectors::Vec3, resources::serialization::{FieldValue, Serialize, StructRepr}};

    
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub(in crate::engine) struct GameObject {
        pub name: String,
        pub position: Vec3,
        pub rotation: Vec3,
        pub scale: Vec3,
        pub components: Vec<StructRepr>,
        pub children: Vec<GameObject>
    }

    impl GameObject {
        pub fn new(world: &World, object: ObjectID) -> Option<GameObject> {
            let object = world.objects.get(object.idx).ok()?;
            let name =  object.name.clone();
            let position =  object.position;
            let rotation =  object.rotation;
            let scale =  object.scale;

            let components = object.components.clone();
            let children = object.children.clone();

            let components = components.into_iter()
                .filter_map(|component| world.components.get(component.index).ok())
                .filter_map(|component| {
                    #[allow(unsafe_code, reason =
                        "Not really that safe, but the game logic loop is single threaded so there is unlikely to be any major issues.
                        This can only really be called from a component body, so borrow() will always fail when deserializing the calling component."
                    )]
                    let component = unsafe { & *component.as_ptr()};
                    let serialize = component.as_serialize()?;

                    Some(serialize.serialize())
                })
                .collect();

            let children = children.into_iter()
                .filter_map(|object| GameObject::new(world, object))
                .collect();

            Some(GameObject {
                name,
                position,
                rotation,
                scale,
                components,
                children,
            })
        }
    }
}