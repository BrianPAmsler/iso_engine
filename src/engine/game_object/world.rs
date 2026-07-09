use std::{any::TypeId, cell::{Ref, RefCell, RefMut}, collections::{BTreeMap, HashSet, VecDeque}, marker::PhantomData, rc::Rc};

use crate::engine::{game_object::game_object::serialize, gl_types::vectors::Vec3, resources::serialization::{DeserializedType, dyn_deserialize}};
use derive_serialize::Serialize;
use itertools::{Either::{Left, Right}, Itertools};
use serde::{Deserialize as _, Serialize as _};

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

type ComponentRef = Rc<RefCell<Box<dyn Component>>>;

pub struct World {
    pub(in crate::engine::game_object) root: ObjectID,
    pub(in crate::engine::game_object) objects: VecAllocator<GameObject>,
    pub(in crate::engine::game_object) components: VecAllocator<ComponentRef>, // TODO: rethink component storage
    ordered_components: BTreeMap<i32, HashSet<ComponentID<()>>>,
    uninitialized_components: BTreeMap<i32, HashSet<ComponentID<()>>>,
    removed_comonents: Vec<(ObjectID, ComponentRef)>,
    removed_objects: Vec<ObjectID>,
    main_camera: Option<Camera>
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ObjectID {
    pub(in crate::engine::game_object) idx: AllocationIndex
}

#[derive(Hash, PartialEq, Eq)]
pub struct ComponentID<T> {
    pub(in crate::engine::game_object) index: AllocationIndex,
    pub(in crate::engine::game_object) owner: ObjectID,
    pub(in crate::engine::game_object) type_: TypeId,
    _pd: PhantomData<T>
}

impl<T> Copy for ComponentID<T> {}
impl<T> Clone for ComponentID<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> ComponentID<T> {
    pub(in crate::engine::game_object) fn transmute<C>(self) -> ComponentID<C> {
        let Self { index, owner, type_, .. } = self;
        ComponentID { index, owner, type_, _pd: PhantomData }
    }
}

impl World {
    pub(in crate::engine) fn new() -> World {
        let mut objects = VecAllocator::new();
        let root = objects.insert(GameObject { name: "root".to_owned(), parent: ObjectID { idx: AllocationIndex::null() }, position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE, components: Vec::new(), children: HashSet::new() });
        let root = ObjectID { idx: root };

        let mut world = World {
            root,
            objects,
            components: VecAllocator::new(),
            ordered_components: BTreeMap::new(),
            uninitialized_components: BTreeMap::new(),
            removed_comonents: Vec::new(),
            removed_objects: Vec::new(),
            main_camera: None
        };

        #[allow(clippy::unwrap_used, reason = "Root is for sure valid.")]
        world.add_component(root, RootComponent::default()).unwrap();

        world
    }

    fn init(engine: &mut Engine) -> Vec<AnyError> {
        // I really hope the compiler can optimize this nonsense
        let components: Vec<ComponentID<_>> = engine.world.uninitialized_components.iter().flat_map(|(_, set)| {
            set.iter().copied()
        }).collect();
        engine.world.uninitialized_components.clear();

        let (components, mut errors): (Vec<_>, Vec<_>) = components.into_iter().map(|component| {
            let owner = component.owner;
            let rc = engine.world.components.get(component.index)?;

            Ok::<(ObjectID, ComponentRef), Error<ComponentError>>((owner, rc.clone()))
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
        let components: Vec<ComponentID<_>> = engine.world.ordered_components.iter().flat_map(|(_, set)| {
            set.iter().copied()
        }).collect();

        let (components, errors): (Vec<_>, Vec<_>) = components.into_iter().map(|component| {
            let owner = component.owner;
            let rc = engine.world.components.get(component.index)?;

            Ok::<(ObjectID, ComponentRef), Error<ComponentError>>((owner, rc.clone()))
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
        let components: Vec<ComponentID<_>> = engine.world.ordered_components.iter().flat_map(|(_, set)| {
            set.iter().copied()
        }).collect();

        let (components, mut errors): (Vec<_>, Vec<_>) = components.into_iter().map(|component| {
            let owner = component.owner;
            let rc = engine.world.components.get(component.index)?;

            Ok::<(ObjectID, ComponentRef), Error<ComponentError>>((owner, rc.clone()))
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

    pub(in crate::engine) fn cleanup(engine: &mut Engine) -> Vec<AnyError> {
        let mut errors = Vec::new();

        for (owner, component) in engine.world.removed_comonents.drain(..).collect_vec() {
            #[allow(clippy::expect_used, reason="Rc should never leak, if it does crashing is justified.")]
            let mut component = Rc::into_inner(component).expect("Cannot remove component due to Rc leak.").into_inner();
            
            if let Err(error) = component.on_remove(engine, owner) {
                errors.push(error);
            }
        }

        for object in engine.world.removed_objects.drain(..) {
            if let Err(error) = engine.world.objects.remove(object.idx) {
                errors.push(error.into());
            }
        }

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
    
    pub fn add_component<C: Component>(&mut self, object: ObjectID, component: C) -> Result<(), ObjectError> {
        let priority = *component.priority();
        let index = self.components.insert(Rc::new(RefCell::new(Box::new(component))));
        let owner = object;
        let object = self.objects.get_mut(object.idx)?;

        let id = ComponentID { index, type_: TypeId::of::<C>(), owner, _pd: PhantomData };
        object.components.push(id);

        let set = self.ordered_components.entry(priority).or_default();
        set.insert(id);

        let uninitialized = self.uninitialized_components.entry(priority).or_default();
        uninitialized.insert(id);

        Ok(())
    }

    pub fn add_component_any(&mut self, object: ObjectID, component: Box<dyn Component>) -> Result<(), ObjectError> {
        let priority = *component.priority();
        let type_ = component.type_id();
        let index = self.components.insert(Rc::new(RefCell::new(component)));
        let owner = object;
        let object = self.objects.get_mut(object.idx)?;

        let id = ComponentID { index, type_, owner, _pd: PhantomData };
        object.components.push(id);

        let set = self.ordered_components.entry(priority).or_default();
        set.insert(id);

        let uninitialized = self.uninitialized_components.entry(priority).or_default();
        uninitialized.insert(id);

        Ok(())
    }

    pub fn remove_component<C>(&mut self, component: ComponentID<C>) -> Result<(), ComponentError> {
        let c = self.components.remove(component.index)?;

        // If the object has been removed already, continue removing component
        match self.objects.get_mut(component.owner.idx) {
            Ok(owner) => {
                owner.components.retain(|e| *e != component.transmute());
            },
            Err(crate::engine::data_structures::error::Error::ElementRemovedError) => (),
            Err(e) => Err(e)?
        }

        if let Some(list) =  self.ordered_components.get_mut(c.borrow().priority()) {
            list.remove(&component.transmute());
        }

        if let Some(list) =  self.uninitialized_components.get_mut(c.borrow().priority()) {
            list.remove(&component.transmute());
        }

        self.removed_comonents.push((component.owner, c));

        Ok(())
    }

    pub fn borrow_component<'a, C: Component>(&'a self, component: ComponentID<C>) -> Result<Ref<'a, C>, ComponentBorrowError> {
        let ref_ = self.components.get(component.index)?.borrow();

        let downcast = Ref::filter_map(ref_, |t| {
            t.downcast_ref()
        }).map_err(|_| ComponentDowncastError { type_name: std::any::type_name::<C>().to_owned() })?;

        Ok(downcast)
    }

    pub fn borrow_component_mut<'a, C: Component>(&'a self, component: ComponentID<C>) -> Result<RefMut<'a, C>, ComponentBorrowError> {
        let ref_ = self.components.get(component.index)?.borrow_mut();

        let downcast = RefMut::filter_map(ref_, |t| {
            t.downcast_mut()
        }).map_err(|_| ComponentDowncastError { type_name: std::any::type_name::<C>().to_owned() })?;

        Ok(downcast)
    }

    /// Creates a GameObject with the given parent.
    /// When parent is ```None``` the object is added to the world's root.
    pub fn create_game_object<S: Into<String>, O: Into<Option<ObjectID>>>(&mut self, name: S, parent: O) -> Result<ObjectID, ObjectError> {
        let parent = parent.into().unwrap_or(self.root);

        // Make sure parent exists before proceding
        self.objects.get(parent.idx)?;

        let name = name.into();
        let new_obj = GameObject { name, parent: self.root, position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE, components: Vec::new(), children: HashSet::new() };
        let new_obj = ObjectID { idx: self.objects.insert(new_obj) };

        self.set_parent(new_obj, Some(parent))?;

        Ok(new_obj)
    }

    pub fn get_component<C: Component>(&self, object: ObjectID) -> Result<Option<ComponentID<C>>, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        for c in obj.components.iter() {
            if c.type_ == TypeId::of::<C>() {
                return Ok(Some(c.transmute()));
            }
        }

        Ok(None)
    }

    pub fn get_components<C: Component>(&self, object: ObjectID) -> Result<Box<[ComponentID<C>]>, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        Ok(obj.components.iter().filter_map(|c| {
            if c.type_ == TypeId::of::<C>() {
                Some(c.transmute())
            } else {
                None
            }
        }).collect())
    }

    pub fn get_children(&self, object: ObjectID) -> Result<Box<[ObjectID]>, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        Ok(obj.children.iter().map(|child| child.to_owned()).collect())
    }

    /// Searches for a child with a given name. When object is ```None``` it will search the root object.
    pub fn find_child<O: Into<Option<ObjectID>>>(&self, object: O, name: &str) -> Result<Option<ObjectID>, ObjectError> {
        let object = object.into().unwrap_or(self.root);

        let obj = self.objects.get(object.idx)?;

        for child in &obj.children {
            let child_name = self.get_name(*child)?;

            if name == child_name {
                return Ok(Some(*child));
            }
        }

        Ok(None)
    }

    /// Searches for a child recursively with a given name. When object is ```None``` it will search the root object.
    pub fn find_child_recursive<O: Into<Option<ObjectID>>>(&self, object: O, name: &str) -> Result<Option<ObjectID>, ObjectError> {
        let object = object.into().unwrap_or(self.root);

        let mut queue = VecDeque::new();
        queue.push_back(object);

        while !queue.is_empty() {
            #[allow(clippy::unwrap_used, reason = "checked")]
            let object = queue.pop_front().unwrap();
            let obj = self.objects.get(object.idx)?;

            for child in &obj.children {
                let child_name = self.get_name(*child)?;

                if name == child_name {
                    return Ok(Some(*child));
                }

                queue.push_back(*child);
            }
        }

        Ok(None)
    }

    /// Returns the ```ObjectID``` of the given object's parent.
    /// Returns none if the parent is the root object.
    pub fn get_parent(&self, object: ObjectID) -> Result<Option<ObjectID>, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        if obj.parent == self.root {
            return Ok(None)
        }

        Ok(Some(obj.parent))
    }

    pub fn set_parent<O: Into<Option<ObjectID>>>(&mut self, object: ObjectID, parent: O) -> Result<(), ObjectError> {
        let parent = parent.into().unwrap_or(self.root);

        self.objects.get(parent.idx)?; // Make sure parent is valid first
        let obj = self.objects.get_mut(object.idx)?;
        let prev_parent = obj.parent;

        // update child parent -> update previous parent's children -> update new parent's children
        obj.parent = parent;

        #[allow(clippy::unwrap_used, reason="ObjectID is already confirmed valid.")]
        let prev_parent = self.objects.get_mut(prev_parent.idx).unwrap();
        prev_parent.children.remove(&object);

        #[allow(clippy::unwrap_used, reason="ObjectID is already confirmed valid.")]
        let new_parent = self.objects.get_mut(parent.idx).unwrap();
        new_parent.children.insert(object);

        Ok(())
    }

    pub fn get_owner<C>(&self, component: ComponentID<C>) -> ObjectID {
        component.owner
    }

    pub fn get_transform(&mut self, object: ObjectID) -> Result<Transform<'_>, ObjectError> {
        let obj = self.objects.get_mut(object.idx)?;

        Ok(Transform { obj })
    }

    pub fn destroy(&mut self, object: ObjectID) -> Result<(), ObjectError> {
        let parent = self.objects.get(object.idx)?.parent;

        self.removed_objects.push(object);

        let parent = self.objects.get_mut(parent.idx)?;
        parent.children.remove(&object);

        let GameObject { components, children, .. } = self.objects.get_mut(object.idx)?;

        let components = components.drain(..).collect_vec();
        let children = children.drain().collect_vec();
        
        #[allow(clippy::unwrap_used, reason="If the code is correct an object's component handles should always be valid.")]
        components.into_iter().try_for_each(|c| {
            self.remove_component(c)
        }).unwrap();

        let results = children.into_iter().map(|child| self.destroy(child)).collect_vec();

        for result in results {
            result?;
        }

        Ok(())
    }

    pub fn serialize_object<S: serde::Serializer, O: Into<Option<ObjectID>>>(&mut self, object: O, serializer: S) -> Result<Option<S::Ok>, S::Error> {
        let object = object.into().unwrap_or(self.root);

        let Some(object) = serialize::GameObject::new(self, object) else { return Ok(None) };

        Ok(Some(object.serialize(serializer)?))
    }

    fn deserialize_object_helper(&mut self, object: serialize::GameObject, target: ObjectID) {
        let Ok(target_obj) = self.objects.get_mut(target.idx) else { return };
        let serialize::GameObject { name, position, rotation, scale, components, children } = object;

        target_obj.name = name;
        target_obj.position = position;
        target_obj.rotation = rotation;
        target_obj.scale = scale;

        let old_components = target_obj.components.iter().copied().collect_vec();
        #[allow(clippy::unwrap_used, reason="If the code is correct an object's component handles should always be valid.")]
        old_components.into_iter().try_for_each(|c| {
            self.remove_component(c)
        }).unwrap();

        for component in components {
            let DeserializedType::Component(component) = dyn_deserialize(component).unwrap() else { unreachable!() };

            self.add_component_any(target, component).unwrap();
        }

        for child in children {
            let new_target = self.create_game_object(child.name.clone(), Some(target)).unwrap();
            self.deserialize_object_helper(child, new_target);
        }
    }

    pub fn deserialize_object<'de, D: serde::Deserializer<'de>, O: Into<Option<ObjectID>>>(&mut self, target: O, deserializer: D) -> Result<Option<()>, D::Error> {
        let target = target.into().unwrap_or(self.root);

        if !self.objects.contains(target.idx) { return Ok(None) }

        let object = serialize::GameObject::deserialize(deserializer)?;

        self.deserialize_object_helper(object, target);

        Ok(Some(()))
    }
}

#[derive(Serialize, Default)]
struct RootComponent {
    main_camera: Option<Camera>
}

impl Component for RootComponent {}