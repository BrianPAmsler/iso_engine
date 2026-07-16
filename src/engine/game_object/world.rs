use std::{any::{Any, TypeId}, borrow::Cow, cell::{Ref, RefCell, RefMut, UnsafeCell}, collections::{BTreeMap, HashSet, VecDeque}, hash::Hash, marker::PhantomData, rc::Rc};

use crate::{engine::{game_object::{component::DynComponent, error::{DeserializeRoot, unions::ObjectDeserializeError}, game_object::serialize}, gl_types::vectors::Vec3, resources::serialization::{AsSerialize, DeserializedType, dyn_deserialize, error::DeserializeError}}, error::MessageErorr};
use derive_serialize::Serialize;
use itertools::Itertools;
use serde::{Deserialize as _, Serialize as _};

use crate::{engine::{Engine, data_structures::{AllocationIndex, VecAllocator}, game_object::{error::{ComponentDowncastError, unions::{ComponentBorrowError, ComponentError, ObjectError}}, game_object::Transform}, graphics::Camera}, error::{Result}};
use crate::error::{any::Error as AnyError};

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

    #[derive(Error, Debug)]
    #[error("Attempted to deserialize non-root object into world root.")]
    pub struct DeserializeRoot;

    pub mod unions {
        use crate::engine::data_structures;
        use crate::engine::game_object::error::{DeadComponent, DeadObject, DeserializeRoot, WorldMismatch};
        use crate::engine::resources::serialization::error::DeserializeError;
        use crate::error::{Error, union};
        use crate::error as errors_module;

        union!(super::DeadComponent, super::WorldMismatch as ComponentError);
        union!(super::DeadObject, super::WorldMismatch as ObjectError);
        union!(ComponentError as RemoveError);
        union!(ComponentError, super::ComponentDowncastError as ComponentBorrowError);
        union!(ObjectError, DeserializeError, DeserializeRoot as ObjectDeserializeError);

        impl From<data_structures::error::Error> for Error<ObjectError> {
            fn from(value: data_structures::error::Error) -> Self {
                match value {
                    data_structures::error::Error::ElementRemovedError => DeadObject.into(),
                    data_structures::error::Error::IndexPointerMismatchError => WorldMismatch.into(),
                }
            }
        }

        impl From<data_structures::error::Error> for ObjectError {
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

        impl From<data_structures::error::Error> for ComponentError {
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

pub(in crate::engine) trait ComponentVec: Any {
    fn check(&self, id: ComponentID<Unknown>) -> std::result::Result<(), ComponentError>;

    fn for_each(&self, f: &mut dyn FnMut(ObjectID, &mut dyn DynComponent, &mut bool));

    fn insert(&mut self, owner: ObjectID, value: Box<dyn DynComponent>) -> ComponentID<Unknown>;

    #[allow(unsafe_code, reason = "Need to bypass borrow check.")]
    unsafe fn get_unchecked(&self, id: ComponentID<Unknown>) -> std::result::Result<&dyn DynComponent, ComponentError>;

    fn remove(&mut self, id: ComponentID<Unknown>) -> std::result::Result<(ObjectID, Box<dyn DynComponent>), ComponentError>;
}

impl<T: Component> ComponentVec for VecAllocator<(ObjectID, RefCell<(bool, T)>)> {
    fn check(&self, id: ComponentID<Unknown>) -> std::result::Result<(), ComponentError> {
        self.get(id.index)?;

        Ok(())
    }

    fn for_each(&self, f: &mut dyn FnMut(ObjectID, &mut dyn DynComponent, &mut bool)) {
        for (_, (id, cell)) in self {
            let mut borrow = cell.borrow_mut();
            let (initialized, value) = &mut *borrow;

            f(*id, value, initialized);
        }
    }

    #[allow(unsafe_code, reason = "Need to bypass borrow check.")]
    unsafe fn get_unchecked(&self, id: ComponentID<Unknown>) -> std::result::Result<&dyn DynComponent, ComponentError> {
        let (_, ref_) = self.get(id.index)?;

        #[allow(unsafe_code, reason = "")]
        let ref_ = & *ref_.as_ptr();
        
        Ok(&ref_.1)
    }
    
    fn insert(&mut self, owner: ObjectID, value: Box<dyn DynComponent>) -> ComponentID<Unknown> {
        let value: Box<dyn Any> = value;

        #[allow(clippy::unwrap_used, reason = "Type should already be checked at this point.")]
        let value: Box<T> = value.downcast().ok().unwrap();
        let index = self.insert((owner, RefCell::new((false, *value))));
        ComponentID { index, owner, type_: ComponentKey::new::<T>(), _pd: PhantomData }
    }

    

    fn remove(&mut self, id: ComponentID<Unknown>) -> std::result::Result<(ObjectID, Box<dyn DynComponent>), ComponentError> {
        let (owner, value) = self.remove(id.index)?;

        Ok((owner, Box::new(value.into_inner().1)))
    }
}

#[derive(Clone, Copy)]
pub(in crate::engine::game_object) struct ComponentKey {
    type_id: TypeId,
    priority: i32
}

impl ComponentKey {
    pub fn new<C: Component>() -> ComponentKey {
        ComponentKey { type_id: TypeId::of::<C>(), priority: C::PRIORITY }
    }

    pub fn new_dyn(component: &dyn DynComponent) -> ComponentKey {
        ComponentKey { type_id: component.type_id(), priority: component.priority() }
    }
}

impl Ord for ComponentKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority).then(self.type_id.cmp(&other.type_id))
    }
}

impl PartialOrd for ComponentKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ComponentKey {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for ComponentKey {}

impl Hash for ComponentKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state)
    }
}

#[derive(PartialEq, Eq)]
pub(in crate::engine) struct Unknown;

impl AsSerialize for Unknown {}
impl Component for Unknown {}

pub struct World {
    pub(in crate::engine::game_object) root: ObjectID,
    pub(in crate::engine::game_object) objects: VecAllocator<GameObject>,
    pub(in crate::engine::game_object) components: BTreeMap<ComponentKey, Rc<UnsafeCell<Box<dyn ComponentVec>>>>,
    components_to_add: Vec<(ObjectID, Box<dyn DynComponent>)>,
    removed_comonents: Vec<ComponentID<Unknown>>,
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
    pub(in crate::engine::game_object) type_: ComponentKey,
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
        let root = objects.insert(GameObject { name: " root ".to_owned(), parent: ObjectID { idx: AllocationIndex::null() }, position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE, components: Vec::new(), children: HashSet::new() });
        let root = ObjectID { idx: root };

        let mut world = World {
            root,
            objects,
            components: BTreeMap::new(),
            components_to_add: Vec::new(),
            removed_comonents: Vec::new(),
            removed_objects: Vec::new(),
            main_camera: None
        };

        #[allow(clippy::unwrap_used, reason = "Root is for sure valid.")]
        world.add_component(root, RootComponent::default()).unwrap();

        world
    }

    pub(in crate::engine) fn update(engine: &mut Engine, delta_time: f32) -> Vec<AnyError> {
        let component_vecs = engine.world.components.values().cloned().collect_vec();

        let mut errors = Vec::new();
        for cell in component_vecs {
            #[allow(unsafe_code, reason = "Each element has its own RefCell, and elements are only inserted and removed after all component updates are processed.")]
            let components = unsafe { &mut *cell.get() };
            components.for_each(&mut |owner, component, initialized| {
                if !*initialized {
                    if let Err(e) = component.init(engine, owner) { errors.push(e) }

                    *initialized = true;
                }

                if let Err(e) = component.update(engine, owner, delta_time) { errors.push(e) }
            });
        }

        errors
    }

    pub(in crate::engine) fn fixed_update(engine: &mut Engine, delta_time: f32) -> Vec<AnyError> {
        let component_vecs = engine.world.components.values().cloned().collect_vec();

        let mut errors = Vec::new();
        for cell in component_vecs {
            #[allow(unsafe_code, reason = "Each element has its own RefCell, and elements are only inserted and removed after all component updates are processed.")]
            let components = unsafe { &mut *cell.get() };
            components.for_each(&mut |owner, component, _| {
                if let Err(e) = component.fixed_update(engine, owner, delta_time) { errors.push(e) }
            });
        }

        errors
    }

    pub(in crate::engine) fn cleanup(engine: &mut Engine) -> Vec<AnyError> {
        let mut errors = Vec::new();

        for (object, component) in engine.world.components_to_add.drain(..) {
            let key = ComponentKey::new_dyn(& *component);

            let entry = if let Some(entry) = engine.world.components.get(&key) {
                entry.get()
            } else {
                let new_vec = if let Some(constructor) = engine.component_vec_constructors.get(&key.type_id) {
                    constructor()
                } else {
                    errors.push(MessageErorr("Attempted to use unregistered component.").into());
                    continue; // Can't use components.entry() because creating our default value might fail, and in that case we need to continue.
                };

                engine.world.components.insert(key, Rc::new(UnsafeCell::new(new_vec)));

                #[allow(clippy::unwrap_used, reason = "get after insert")]
                engine.world.components.get(&key).unwrap().get()
            };

            #[allow(unsafe_code, reason = "Each element has its own RefCell, and elements are only inserted and removed after all component updates are processed.")]
            let entry = unsafe { &mut *entry };
            let id = entry.insert(object, component);

            if let Ok(object) = engine.world.objects.get_mut(object.idx) {
                object.components.push(id);
            }
        }

        for component in engine.world.removed_comonents.drain(..).collect_vec() {
            #[allow(clippy::unwrap_used, reason = "Key must exist if ComponentID exists.")]
            let cell = engine.world.components.get(&component.type_).unwrap();
            #[allow(unsafe_code, reason = "Each element has its own RefCell, and elements are only inserted and removed after all component updates are processed.")]
            let components = unsafe { &mut *cell.get() };

            let (owner, mut component) = match components.remove(component) {
                Ok(v) => v,
                Err(error) => {
                    errors.push(error.into());
                    continue;
                }
            };
            
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
        self.add_dyn_component(object, Box::new(component))
    }

    pub(in crate::engine) fn add_dyn_component(&mut self, object: ObjectID, component: Box<dyn DynComponent>) -> Result<(), ObjectError> {
        // Make sure object is valid first
        self.objects.get(object.idx)?;

        self.components_to_add.push((object, component));

        Ok(())
    }

    pub fn remove_component<C: Component>(&mut self, component: ComponentID<C>) -> Result<(), ComponentError> {
        #[allow(clippy::unwrap_used, reason = "Key must exist if ComponentID<C> exists.")]
        let cell = self.components.get(&ComponentKey::new::<C>()).unwrap();
        #[allow(unsafe_code, reason = "Each element has its own RefCell, and elements are only inserted and removed after all component updates are processed.")]
        let components = unsafe { &mut *cell.get() };
        components.check(component.transmute())?;

        // If the object has been removed already, continue removing component
        match self.objects.get_mut(component.owner.idx) {
            Ok(owner) => {
                owner.components.retain(|e| *e != component.transmute());
            },
            Err(crate::engine::data_structures::error::Error::ElementRemovedError) => (),
            Err(e) => Err(e)?
        }

        self.removed_comonents.push(component.transmute());

        Ok(())
    }

    pub fn borrow_component<'a, C: Component>(&'a self, component: ComponentID<C>) -> Result<Ref<'a, C>, ComponentBorrowError> {
        #[allow(clippy::unwrap_used, reason = "Key must exist if ComponentID<C> exists.")]
        let cell = self.components.get(&ComponentKey::new::<C>()).unwrap();
        #[allow(unsafe_code, reason = "Each element has its own RefCell, and elements are only inserted and removed after all component updates are processed.")]
        let components: &mut Box<dyn ComponentVec> = unsafe { &mut *cell.get() };
        let components: &mut dyn Any = &mut *components;
        
        let downcast: &VecAllocator<(ObjectID, RefCell<(bool, C)>)> = components.downcast_ref().ok_or(ComponentDowncastError { type_name: std::any::type_name::<C>().to_owned() })?;

        let (_, ref_) = downcast.get(component.index)?;

        let ref_ = Ref::map(ref_.borrow(), |(_, value)| value);

        Ok(ref_)
    }

    pub fn borrow_component_mut<'a, C: Component>(&'a self, component: ComponentID<C>) -> Result<RefMut<'a, C>, ComponentBorrowError> {
        #[allow(clippy::unwrap_used, reason = "Key must exist if ComponentID<C> exists.")]
        let cell = self.components.get(&ComponentKey::new::<C>()).unwrap();
        #[allow(unsafe_code, reason = "Each element has its own RefCell, and elements are only inserted and removed after all component updates are processed.")]
        let components: &mut Box<dyn ComponentVec> = unsafe { &mut *cell.get() };
        let components: &mut dyn Any = &mut *components;
        
        let downcast: &VecAllocator<(ObjectID, RefCell<(bool, C)>)> = components.downcast_ref().ok_or(ComponentDowncastError { type_name: std::any::type_name::<C>().to_owned() })?;

        let (_, ref_) = downcast.get(component.index)?;

        let ref_ = RefMut::map(ref_.borrow_mut(), |(_, value)| value);

        Ok(ref_)
    }

    /// Creates a GameObject with the given parent.
    /// When parent is ```None``` the object is added to the world's root.
    pub fn create_game_object<S: Into<String>, O: Into<Option<ObjectID>>>(&mut self, name: S, parent: O) -> Result<ObjectID, ObjectError> {
        let parent = parent.into().unwrap_or(self.root);

        // Make sure parent exists before proceding
        self.objects.get(parent.idx)?;

        let mut name = name.into();

        let trimmed = name.trim();

        if trimmed != name {
            name = trimmed.to_owned();
        }

        let new_obj = GameObject { name, parent: self.root, position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE, components: Vec::new(), children: HashSet::new() };
        let new_obj = ObjectID { idx: self.objects.insert(new_obj) };

        self.set_parent(new_obj, Some(parent))?;

        Ok(new_obj)
    }

    pub fn get_component<C: Component>(&self, object: ObjectID) -> Result<Option<ComponentID<C>>, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        for c in obj.components.iter() {
            if c.type_.type_id == TypeId::of::<C>() {
                return Ok(Some(c.transmute()));
            }
        }

        Ok(None)
    }

    pub fn get_components<C: Component>(&self, object: ObjectID) -> Result<Box<[ComponentID<C>]>, ObjectError> {
        let obj = self.objects.get(object.idx)?;

        Ok(obj.components.iter().filter_map(|c| {
            if c.type_.type_id == TypeId::of::<C>() {
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

    fn deserialize_object_helper(&mut self, object: serialize::GameObject, target: ObjectID) -> Result<(), ObjectDeserializeError> {
        let target_obj = self.objects.get_mut(target.idx).map_err(Into::<ObjectError>::into)?;
        let serialize::GameObject { name, position, rotation, scale, components, children } = object;

        if target == self.root && name != " root " {
            return Err(DeserializeRoot)?;
        }

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
            let DeserializedType::Component(component) = dyn_deserialize(component)? else { unreachable!() };

            self.add_dyn_component(target, component)?;
        }

        for child in children {
            let new_target = self.create_game_object(child.name.clone(), Some(target))?;
            self.deserialize_object_helper(child, new_target)?;
        }

        Ok(())
    }

    pub fn deserialize_object<'de, D: serde::Deserializer<'de>, O: Into<Option<ObjectID>>>(&mut self, target: O, deserializer: D) -> Result<Result<(), ObjectDeserializeError>, D::Error> {
        let target = target.into().unwrap_or(self.root);

        if let Err(error) = self.objects.get(target.idx) {
            return Ok(Err(ObjectError::from(error).into()));
        }

        let object = serialize::GameObject::deserialize(deserializer)?;

        Ok(self.deserialize_object_helper(object, target))
    }
}

#[derive(Serialize, Default)]
pub(in crate::engine) struct RootComponent {
    main_camera: Option<Camera>
}

impl Component for RootComponent {}