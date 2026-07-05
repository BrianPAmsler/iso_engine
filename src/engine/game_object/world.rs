use std::{any::TypeId, cell::{Ref, RefCell, RefMut}, collections::{BTreeMap, HashSet}, rc::Rc};

use crate::engine::gl_types::vectors::Vec3;
use itertools::{Either::{Left, Right}, Itertools};

use crate::{engine::{Engine, data_structures::{AllocationIndex, VecAllocator}, game_object::{error::{ComponentDowncastError, unions::{ComponentBorrowError, ComponentError, ObjectError}}, game_object::Transform}, graphics::Camera}, error::{Result}};
use crate::error::{Error, any::Error as AnyError};

use super::{component::Component, game_object::GameObject};

pub mod error {
    use error::Error;

    #[derive(Error, Debug)]
    #[error("Component is dead!")]
    pub struct DeadComponent;

    #[derive(Error, Debug)]
    #[error("Object is dead!")]
    pub struct DeadObject;

    #[derive(Error, Debug)]
    #[error("Object must belong to the same world!")]
    pub struct WorldMismatch;

    #[derive(Error, Debug)]
    #[error("Component is not of type {type_name}")]
    pub struct ComponentDowncastError { pub type_name: String }

    pub mod unions {
        use crate::engine::data_structures;
        use crate::engine::game_object::error::{DeadComponent, DeadObject, WorldMismatch};
        use crate::error::{Error, union};
        use crate::error as errors_module;

        union!(super::DeadComponent, super::WorldMismatch as ComponentError);
        union!(super::DeadObject, super::WorldMismatch as ObjectError);
        union!(ComponentError as RemoveError);
        union!(ComponentError, super::ComponentDowncastError as ComponentBorrowError);

        impl From<data_structures::error::Error> for Error<ObjectError> {
            fn from(value: data_structures::error::Error) -> Self {
                match value {
                    data_structures::error::Error::ElementRemovedError => DeadObject.into(),
                    data_structures::error::Error::IndexPointerMismatchError => WorldMismatch.into(),
                }
            }
        }

        impl From<data_structures::error::Error> for Error<ComponentError> {
            fn from(value: data_structures::error::Error) -> Self {
                match value {
                    data_structures::error::Error::ElementRemovedError => DeadComponent.into(),
                    data_structures::error::Error::IndexPointerMismatchError => WorldMismatch.into(),
                }
            }
        }

        impl From<data_structures::error::Error> for Error<ComponentBorrowError> {
            fn from(value: data_structures::error::Error) -> Self {
                match value {
                    data_structures::error::Error::ElementRemovedError => ComponentError::DeadComponent(DeadComponent).into(),
                    data_structures::error::Error::IndexPointerMismatchError => ComponentError::WorldMismatch(WorldMismatch).into(),
                }
            }
        }
    }
}

pub struct World {
    pub(in crate::engine::game_object) root: ObjectID,
    pub(in crate::engine::game_object) objects: VecAllocator<GameObject>,
    pub(in crate::engine::game_object) components: VecAllocator<Rc<RefCell<Box<dyn Component>>>>, // TODO: rethink component storage
    ordered_components: BTreeMap<i32, HashSet<ComponentID>>,
    uninitialized_components: BTreeMap<i32, HashSet<ComponentID>>,
    removed_comonents: Vec<(ObjectID, Rc<RefCell<Box<dyn Component>>>)>,
    main_camera: Option<Camera>
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ObjectID {
    idx: AllocationIndex
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ComponentID {
    index: AllocationIndex,
    owner: ObjectID,
    type_: TypeId
}

impl World {
    pub(in crate::engine) fn new() -> World {
        let mut objects = VecAllocator::new();
        let root = objects.insert(GameObject { name: "root".to_owned(), parent: ObjectID { idx: AllocationIndex::null() }, position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE, components: Vec::new(), children: HashSet::new() });
        let root = ObjectID { idx: root };

        World {
            root,
            objects,
            components: VecAllocator::new(),
            ordered_components: BTreeMap::new(),
            uninitialized_components: BTreeMap::new(),
            removed_comonents: Vec::new(),
            main_camera: None
        }
    }

    fn init(engine: &mut Engine) -> Vec<AnyError> {
        // I really hope the compiler can optimize this nonsense
        let components: Vec<ComponentID> = engine.world.uninitialized_components.iter().flat_map(|(_, set)| {
            set.iter().cloned()
        }).collect();
        engine.world.uninitialized_components.clear();

        let (components, mut errors): (Vec<_>, Vec<_>) = components.into_iter().map(|component| {
            let owner = component.owner;
            let rc = engine.world.components.get(component.index)?;

            Ok::<(ObjectID, Rc<RefCell<Box<dyn Component>>>), Error<ComponentError>>((owner, rc.clone()))
        })
        .partition_map(|result| match result {
            Ok(component) => Left(component),
            Err(err) => Right(err.into()),
        });

        errors.extend(
            components.into_iter().filter_map(|(owner, rc)| {
                rc.borrow_mut().init(engine, owner).err()
            })
        );

        errors
    }

    pub(in crate::engine) fn update(engine: &mut Engine, delta_time: f32) -> Vec<AnyError> {
        // I really hope the compiler can optimize this nonsense
        let mut init_errors = Self::init(engine);

        // I really hope the compiler can optimize this nonsense
        let components: Vec<ComponentID> = engine.world.ordered_components.iter().flat_map(|(_, set)| {
            set.iter().cloned()
        }).collect();

        let (components, errors): (Vec<_>, Vec<_>) = components.into_iter().map(|component| {
            let owner = component.owner;
            let rc = engine.world.components.get(component.index)?;

            Ok::<(ObjectID, Rc<RefCell<Box<dyn Component>>>), Error<ComponentError>>((owner, rc.clone()))
        })
        .partition_map(|result| match result {
            Ok(component) => Left(component),
            Err(err) => Right(err.into()),
        });

        init_errors.extend(errors);
        let mut errors = init_errors;

        errors.extend(
            components.into_iter().filter_map(|(owner, rc)| {
                rc.borrow_mut().update(engine, owner, delta_time).err()
            })
        );

        errors
    }

    pub(in crate::engine) fn fixed_update(engine: &mut Engine, delta_time: f32) -> Vec<AnyError> {
        // I really hope the compiler can optimize this nonsense
        let components: Vec<ComponentID> = engine.world.ordered_components.iter().flat_map(|(_, set)| {
            set.iter().cloned()
        }).collect();

        let (components, mut errors): (Vec<_>, Vec<_>) = components.into_iter().map(|component| {
            let owner = component.owner;
            let rc = engine.world.components.get(component.index)?;

            Ok::<(ObjectID, Rc<RefCell<Box<dyn Component>>>), Error<ComponentError>>((owner, rc.clone()))
        })
        .partition_map(|result| match result {
            Ok(component) => Left(component),
            Err(err) => Right(err.into()),
        });

        errors.extend(
            components.into_iter().filter_map(|(owner, rc)| {
                rc.borrow_mut().fixed_update(engine, owner, delta_time).err()
            })
        );

        errors
    }

    pub fn get_main_camera(&self) -> Option<&Camera> {
        self.main_camera.as_ref()
    }

    pub fn get_main_camera_mut(&mut self) -> Option<&mut Camera> {
        self.main_camera.as_mut()
    }

    pub fn set_main_camera(&mut self, camera: Camera) {
        self.main_camera = Some(camera)
    }

    pub fn get_name(&self, object: ObjectID) -> Result<&str, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        Ok(&obj.name)
    }

    pub fn set_name(&mut self, object: ObjectID, name: String) -> Result<(), ObjectError> {
        let obj = self.objects.get_mut(object.idx)?;

        obj.name = name;

        Ok(())
    }

    pub fn get_root(&self) -> ObjectID {
        self.root
    }

    pub fn add_component<C: Component>(&mut self, object: ObjectID, component: C) -> Result<(), ObjectError> {
        let priority = *component.priority();
        let index = self.components.insert(Rc::new(RefCell::new(Box::new(component))));
        let owner = object;
        let object = self.objects.get_mut(object.idx)?;

        let id = ComponentID { index, type_: TypeId::of::<C>(), owner };
        object.components.push(id);

        let set = self.ordered_components.entry(priority).or_default();
        set.insert(id);

        let uninitialized = self.uninitialized_components.entry(priority).or_default();
        uninitialized.insert(id);

        Ok(())
    }

    pub fn add_component_any(&mut self, object: ObjectID, component: Box<dyn Component>) -> Result<(), ObjectError> {
        let priority = *component.priority();
        let type_ = (*component).type_id();
        let index = self.components.insert(Rc::new(RefCell::new(component)));
        let owner = object;
        let object = self.objects.get_mut(object.idx)?;

        let id = ComponentID { index, type_, owner };
        object.components.push(id);

        let set = self.ordered_components.entry(priority).or_default();
        set.insert(id);

        let uninitialized = self.uninitialized_components.entry(priority).or_default();
        uninitialized.insert(id);

        Ok(())
    }

    pub fn remove_component(&mut self, component: ComponentID) -> Result<(), ComponentError> {
        let c = self.components.remove(component.index)?;

        match self.ordered_components.get_mut(c.borrow().priority()) {
            Some(list) => list.remove(&component),
            None => unreachable!(),
        };

        self.removed_comonents.push((component.owner, c));

        Ok(())
    }

    pub fn borrow_component<'a, C: Component>(&'a self, component: ComponentID) -> Result<Ref<'a, C>, ComponentBorrowError> {
        let ref_ = self.components.get(component.index)?.borrow();

        let downcast = Ref::filter_map(ref_, |t| {
            t.downcast_ref()
        }).map_err(|_| ComponentDowncastError { type_name: std::any::type_name::<C>().to_owned() })?;

        Ok(downcast)
    }

    pub fn borrow_component_mut<'a, C: Component>(&'a self, component: ComponentID) -> Result<RefMut<'a, C>, ComponentBorrowError> {
        let ref_ = self.components.get(component.index)?.borrow_mut();

        let downcast = RefMut::filter_map(ref_, |t| {
            t.downcast_mut()
        }).map_err(|_| ComponentDowncastError { type_name: std::any::type_name::<C>().to_owned() })?;

        Ok(downcast)
    }

    pub fn create_game_object<S: Into<String>>(&mut self, name: S, parent: ObjectID) -> Result<ObjectID, ObjectError> {
        self.objects.get(parent.idx)?;

        let name = name.into();
        let new_obj = GameObject { name, parent: self.root, position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE, components: Vec::new(), children: HashSet::new() };
        let new_obj = ObjectID { idx: self.objects.insert(new_obj) };

        self.set_parent(new_obj, parent)?;

        Ok(new_obj)
    }

    pub fn get_component<C: Component>(&self, object: ObjectID) -> Result<Option<ComponentID>, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        for c in obj.components.iter() {
            if c.type_ == TypeId::of::<C>() {
                return Ok(Some(*c));
            }
        }

        Ok(None)
    }

    pub fn get_components<C: Component>(&self, object: ObjectID) -> Result<Box<[ComponentID]>, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        Ok(obj.components.iter().filter_map(|c| {
            if c.type_ == TypeId::of::<C>() {
                Some(c.to_owned())
            } else {
                None
            }
        }).collect())
    }

    pub fn get_children(&self, object: ObjectID) -> Result<Box<[ObjectID]>, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        Ok(obj.children.iter().map(|child| child.to_owned()).collect())
    }

    pub fn find_child(&self, object: ObjectID, name: &str) -> Result<Option<ObjectID>, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        for child in &obj.children {
            let child_name = self.get_name(*child)?;

            if name == child_name {
                return Ok(Some(*child));
            }
        }

        Ok(None)
    }

    pub fn get_parent(&self, object: ObjectID) -> Result<ObjectID, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        Ok(obj.parent)
    }

    pub fn set_parent(&mut self, object: ObjectID, parent: ObjectID) -> Result<(), ObjectError> {
        self.objects.get(parent.idx)?; // Make sure parent is valid first
        let obj = self.objects.get_mut(object.idx)?;
        let prev_parent = obj.parent;

        // update child parent -> update previous parent's children -> update new parent's children
        obj.parent = parent;

        let prev_parent = self.objects.get_mut(prev_parent.idx).unwrap(); // This should already be valid so unwrap
        prev_parent.children.remove(&object);

        let new_parent = self.objects.get_mut(parent.idx).unwrap();
        new_parent.children.insert(object);

        Ok(())
    }

    pub fn get_owner(&self, component: ComponentID) -> ObjectID {
        component.owner
    }

    pub fn get_transform(&mut self, object: ObjectID) -> Result<Transform<'_>, ObjectError> {
        let obj = self.objects.get_mut(object.idx)?;

        Ok(Transform { obj })
    }

    pub fn destroy(&mut self, object: ObjectID) -> Result<(), ObjectError> {
        let obj = self.objects.get(object.idx)?;

        let parent = self.objects.get_mut(obj.parent.idx).unwrap(); // This should already be valid so unwrap
        parent.children.remove(&object);

        self.objects.remove(object.idx)?;

        Ok(())
    }

    pub(in crate::engine) fn get_removed_components(&mut self) -> Vec<(ObjectID, Rc<RefCell<Box<dyn Component>>>)> {
        std::mem::take(&mut self.removed_comonents)
    }
}

// fn obj_error(error: crate::engine::data_structures::error::Error) -> ObjectError {
//     match error {
//         crate::engine::data_structures::error::Error::ElementRemovedError => DeadObject {}.into(),
//         crate::engine::data_structures::error::Error::IndexPointerMismatchError => WorldMismatch { other: "" }.into(),
//     }
// }

// fn comp_error(error: crate::engine::data_structures::error::Error) -> ComponentError {
//     match error {
//         crate::engine::data_structures::error::Error::ElementRemovedError => DeadComponent {}.into(),
//         crate::engine::data_structures::error::Error::IndexPointerMismatchError => WorldMismatch { other: "" }.into(),
//     }
// }