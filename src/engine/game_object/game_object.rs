use std::collections::HashSet;

use crate::engine::gl_types::vectors::Vec3;

use super::{ComponentID, ObjectID};

pub struct Transform<'a> {
    pub(in crate::engine) obj: &'a mut GameObject
}

#[derive(Clone)]
pub struct GameObject {
    pub(in crate::engine) name: String,
    pub(in crate::engine) parent: ObjectID,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub(in crate::engine) components: Vec<ComponentID<()>>,
    pub(in crate::engine) children: HashSet<ObjectID>,
    pub(in crate::engine) bounding_box: Option<(Vec3, Vec3)>
}

pub(in crate::engine::game_object) mod serialize {
    use crate::engine::{game_object::{ObjectID, World}, gl_types::vectors::Vec3, resources::serialization::{StructRepr}};

    
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