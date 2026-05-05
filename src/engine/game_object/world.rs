use std::{any::TypeId, cell::{Ref, RefCell, RefMut}, collections::{BTreeMap, HashSet}, rc::Rc};

use crate::engine::{Engine, data_structures::{AllocationIndex, VecAllocator}, errors::{ObjectError, Result}, graphics::Camera};

use super::{component::{components::Transform, Component}, game_object::GameObject};

pub struct World {
    pub(in crate::engine::game_object) root: ObjectID,
    pub(in crate::engine::game_object) objects: VecAllocator<GameObject>,
    pub(in crate::engine::game_object) components: VecAllocator<Rc<RefCell<Box<dyn Component>>>>, // TODO: rethink component storage
    ordered_components: BTreeMap<i32, HashSet<ComponentID>>,
    uninitialized_components: BTreeMap<i32, HashSet<ComponentID>>,
    removed_comonents: Vec<(ObjectID, Box<dyn Component>)>,
    main_camera: RefCell<Option<Rc<RefCell<Camera>>>> // yikes
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
        let root = objects.insert(GameObject { name: "root".to_owned(), parent: ObjectID { idx: AllocationIndex::null() }, components: Vec::new(), children: HashSet::new() });
        let root = ObjectID { idx: root };

        let mut world = World {
            root,
            objects,
            components: VecAllocator::new(),
            ordered_components: BTreeMap::new(),
            uninitialized_components: BTreeMap::new(),
            removed_comonents: Vec::new(),
            main_camera: RefCell::new(None)
        };

        world.add_component(world.root, Transform::ZERO).expect("This also shouldn't happen!");

        world
    }

    fn init(engine: &mut Engine) -> Result<()> {
        // I really hope the compiler can optimize this nonsense
        let components: Vec<ComponentID> = engine.world.uninitialized_components.iter().flat_map(|(_, set)| {
            set.iter().cloned()
        }).collect();
        engine.world.uninitialized_components.clear();

        let components: Vec<(ObjectID, Rc<RefCell<Box<dyn Component>>>)> = components.into_iter().map(|component| {
            let owner = component.owner;
            let rc = engine.world.components.get(component.index).map_err(comp_error)?;

            Ok::<(ObjectID, Rc<RefCell<Box<dyn Component>>>), ObjectError>((owner, rc.clone()))
        }).collect::<std::result::Result<Vec<_>, ObjectError>>()?;

        components.into_iter().try_for_each(|(owner, rc)| {
            rc.borrow_mut().init(engine, owner)?; 

            Ok(())
        })
    }

    pub(in crate::engine) fn update(engine: &mut Engine, delta_time: f32) -> Result<()> {
        // I really hope the compiler can optimize this nonsense
        Self::init(engine)?;

        // I really hope the compiler can optimize this nonsense
        let components: Vec<ComponentID> = engine.world.ordered_components.iter().flat_map(|(_, set)| {
            set.iter().cloned()
        }).collect();

        let components: Vec<(ObjectID, Rc<RefCell<Box<dyn Component>>>)> = components.into_iter().map(|component| {
            let owner = component.owner;
            let rc = engine.world.components.get(component.index).map_err(comp_error)?;

            Ok::<(ObjectID, Rc<RefCell<Box<dyn Component>>>), ObjectError>((owner, rc.clone()))
        }).collect::<std::result::Result<Vec<_>, ObjectError>>()?;

        components.into_iter().try_for_each(|(owner, rc)| {
            rc.borrow_mut().update(engine, owner, delta_time)?; 

            Ok(())
        })
    }

    pub(in crate::engine) fn fixed_update(engine: &mut Engine, delta_time: f32) -> Result<()> {
        // I really hope the compiler can optimize this nonsense
        let components: Vec<ComponentID> = engine.world.ordered_components.iter().flat_map(|(_, set)| {
            set.iter().cloned()
        }).collect();

        let components: Vec<(ObjectID, Rc<RefCell<Box<dyn Component>>>)> = components.into_iter().map(|component| {
            let owner = component.owner;
            let rc = engine.world.components.get(component.index).map_err(comp_error)?;

            Ok::<(ObjectID, Rc<RefCell<Box<dyn Component>>>), ObjectError>((owner, rc.clone()))
        }).collect::<std::result::Result<Vec<_>, ObjectError>>()?;

        components.into_iter().try_for_each(|(owner, rc)| {
            rc.borrow_mut().fixed_update(engine, owner, delta_time)?; 

            Ok(())
        })
    }

    pub fn get_main_camera(&self) -> Option<Rc<RefCell<Camera>>> {
        self.main_camera.borrow().clone()
    }

    pub fn set_main_camera(&self, camera: Rc<RefCell<Camera>>) {
        *self.main_camera.borrow_mut() = Some(camera)
    }

    pub fn get_name(&self, object: ObjectID) -> Result<&str> {
        let obj = self.objects.get(object.idx).map_err(obj_error)?;

        Ok(&obj.name)
    }

    pub fn set_name(&mut self, object: ObjectID, name: String) -> Result<()> {
        let obj = self.objects.get_mut(object.idx).map_err(obj_error)?;

        obj.name = name;

        Ok(())
    }

    pub fn get_root(&self) -> ObjectID {
        self.root
    }

    pub fn add_component<C: Component>(&mut self, object: ObjectID, component: C) -> Result<()> {
        let priority = *component.priority();
        let index = self.components.insert(Rc::new(RefCell::new(Box::new(component))));
        let owner = object;
        let object = self.objects.get_mut(object.idx).map_err(obj_error)?;

        let id = ComponentID { index, type_: TypeId::of::<C>(), owner };
        object.components.push(id);

        let set = self.ordered_components.entry(priority).or_insert(HashSet::new());
        set.insert(id);

        let uninitialized = self.uninitialized_components.entry(priority).or_insert(HashSet::new());
        uninitialized.insert(id);

        Ok(())
    }

    pub fn remove_component(&mut self, component: ComponentID) -> Result<()> {
        let c = self.components.remove(component.index).map_err(comp_error)?;
        let c = Rc::into_inner(c).ok_or("Component still owned somewhere.")?.into_inner();

        match self.ordered_components.get_mut(c.priority()) {
            Some(list) => list.remove(&component),
            None => unreachable!(),
        };

        self.removed_comonents.push((component.owner, c));

        Ok(())
    }

    pub fn borrow_component<'a, C: Component>(&'a self, component: ComponentID) -> Result<Ref<'a, C>> {
        let ref_ = self.components.get(component.index).map_err(obj_error)?.borrow();

        let downcast = Ref::filter_map(ref_, |t| {
            t.downcast_ref()
        }).map_err(|_| ObjectError::ComponentDowncastError { type_name: std::any::type_name::<C>().to_owned() })?;

        Ok(downcast)
    }

    pub fn borrow_component_mut<'a, C: Component>(&'a self, component: ComponentID) -> Result<RefMut<'a, C>> {
        let ref_ = self.components.get(component.index).map_err(obj_error)?.borrow_mut();

        let downcast = RefMut::filter_map(ref_, |t| {
            t.downcast_mut()
        }).map_err(|_| ObjectError::ComponentDowncastError { type_name: std::any::type_name::<C>().to_owned() })?;

        Ok(downcast)
    }

    pub fn create_game_object<S: Into<String>>(&mut self, name: S, parent: ObjectID) -> Result<ObjectID> {
        self.objects.get(parent.idx).map_err(obj_error)?;

        let name = name.into();
        let new_obj = GameObject { name, parent: self.root, components: Vec::new(), children: HashSet::new() };
        let new_obj = ObjectID { idx: self.objects.insert(new_obj) };

        self.add_component(new_obj, Transform::ZERO).expect("This shouldn't happen!");
        self.set_parent(new_obj, parent).unwrap();

        Ok(new_obj)
    }

    pub fn get_component<C: Component>(&self, object: ObjectID) -> Result<ComponentID> {
        let obj = self.objects.get(object.idx).map_err(obj_error)?;

        for c in obj.components.iter() {
            if c.type_ == TypeId::of::<C>() {
                return Ok(c.clone());
            }
        }

        Err(ObjectError::ComponentNotFoundError)?
    }

    pub fn get_components<C: Component>(&self, object: ObjectID) -> Result<Box<[ComponentID]>> {
        let obj = self.objects.get(object.idx).map_err(obj_error)?;

        Ok(obj.components.iter().filter_map(|c| {
            if c.type_ == TypeId::of::<C>() {
                Some(c.to_owned())
            } else {
                None
            }
        }).collect())
    }

    pub fn get_children(&self, object: ObjectID) -> Result<Box<[ObjectID]>> {
        let obj = self.objects.get(object.idx).map_err(obj_error)?;

        Ok(obj.children.iter().map(|child| child.to_owned()).collect())
    }

    pub fn find_child(&self, object: ObjectID, name: &str) -> Result<Option<ObjectID>> {
        let obj = self.objects.get(object.idx).map_err(obj_error)?;

        for child in &obj.children {
            let child_name = self.get_name(*child).unwrap();

            if name == child_name {
                return Ok(Some(*child));
            }
        }

        Ok(None)
    }

    pub fn get_parent(&self, object: ObjectID) -> Result<ObjectID> {
        let obj = self.objects.get(object.idx).map_err(obj_error)?;

        Ok(obj.parent)
    }

    pub fn set_parent(&mut self, object: ObjectID, parent: ObjectID) -> Result<()> {
        self.objects.get(parent.idx).map_err(obj_error)?; // Make sure parent is valid first
        let obj = self.objects.get_mut(object.idx).map_err(obj_error)?;
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

    pub fn destroy(&mut self, object: ObjectID) -> Result<()> {
        let obj = self.objects.get(object.idx).map_err(obj_error)?;

        let parent = self.objects.get_mut(obj.parent.idx).unwrap(); // This should already be valid so unwrap
        parent.children.remove(&object);

        self.objects.remove(object.idx).map_err(obj_error)?;

        Ok(())
    }

    pub(in crate::engine) fn get_removed_components(&mut self) -> Vec<(ObjectID, Box<dyn Component>)> {
        let removed = std::mem::replace(&mut self.removed_comonents, Vec::new());

        removed
    }
}

fn obj_error(error: crate::engine::data_structures::error::Error) -> ObjectError {
    match error {
        crate::engine::data_structures::error::Error::ElementRemovedError => ObjectError::DeadObjectError,
        crate::engine::data_structures::error::Error::IndexPointerMismatchError => ObjectError::WorldMismatchError { other: "" },
    }
}

fn comp_error(error: crate::engine::data_structures::error::Error) -> ObjectError {
    match error {
        crate::engine::data_structures::error::Error::ElementRemovedError => ObjectError::DeadComponentError,
        crate::engine::data_structures::error::Error::IndexPointerMismatchError => ObjectError::WorldMismatchError { other: "" },
    }
}