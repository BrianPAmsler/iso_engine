use std::{any::Any, collections::HashMap, io::ErrorKind::NotFound, marker::PhantomData, path::{Path, PathBuf}, sync::{Arc, OnceLock, RwLock, Weak}};

use itertools::Itertools;
use resource_packager::packager::ResourcePackagerError;

use crate::{engine::resources::{error::{InvalidDowncast, LoadError, ResourceError, ResourceLoadError}, pack::AssetPack}, error::Result};

pub struct ResourceHandle<T: ?Sized + 'static, E: std::error::Error + ?Sized> {
    data: Arc<ResourceData>,
    status: Arc<RwLock<ResourceStatus>>,
    type_name: &'static str,
    error_name: &'static str,
    _type: PhantomData<T>,
    _error: PhantomData<E>
}

impl<T, E: std::error::Error + Clone + 'static> ResourceHandle<T, E> {
    fn new() -> ResourceHandle<T, E>  {
        ResourceHandle {
            data: Arc::new(OnceLock::new()),
            status: Arc::new(RwLock::new(ResourceStatus::Unloaded)),
            type_name: std::any::type_name::<T>(),
            error_name: std::any::type_name::<E>(),
            _type: PhantomData,
            _error: PhantomData
        }
    }

    fn downgrade(&self) -> ResourceHandleWeak<T, E> {
        let Self { data, status, type_name, error_name, _type, _error } = &self;
        let (type_name, error_name, _type, _error) = (*type_name, *error_name, *_type, *_error);

        let data = Arc::downgrade(data);
        let status = Arc::downgrade(status);

        ResourceHandleWeak { data, status, type_name, error_name, _type, _error }
    }

    pub fn value(&self) -> Option<std::result::Result<&T, ResourceLoadError<E>>> {
        self.data.get()
            .map(|result| {
                result.as_ref()
                    .map_err(|err| {
                        let err: Option<ResourceLoadError<E>> = err.downcast_and_clone();
                        let to = std::any::type_name::<E>();
                        let from = self.error_name;
                        match err {
                            Some(value) => value,
                            None => ResourceLoadError::InvalidDowncast(InvalidDowncast { to, from }),
                        }
                    })
                    .and_then(|value| {
                        let value: Option<&T> = value.downcast_ref();
                        let from = std::any::type_name::<T>();
                        let to = self.type_name;
                        Ok(value.ok_or(InvalidDowncast { to, from })?)
                    })
            })
    }

    pub fn is_loaded(&self) -> bool {
        self.data.get().is_some()
    }

    pub fn can_take(&self) -> bool {
        Arc::strong_count(&self.data) == 1
    }

    pub fn take(self) -> std::result::Result<Option<std::result::Result<T, ResourceLoadError<E>>>, Self> {
        if Arc::strong_count(&self.data) > 1 { return Err(self) };
        let inner = Arc::into_inner(self.data).unwrap();
        let result = inner.into_inner()
            .map(|result| {
                result
                    .map_err(|err| {
                        let err: Option<ResourceLoadError<E>> = err.downcast();
                        let to = std::any::type_name::<E>();
                        let from = self.error_name;
                        match err {
                            Some(value) => value,
                            None => ResourceLoadError::InvalidDowncast(InvalidDowncast { to, from }),
                        }
                    })
                    .and_then(|value| {
                        let value: Option<Box<T>> = value.downcast().ok();
                        let value = value.map(|box_| *box_);
                        let from = std::any::type_name::<T>();
                        let to = self.type_name;
                        Ok(value.ok_or(InvalidDowncast { to, from })?)
                    })
            });
        
        Ok(result)
    }

    pub fn share(self) -> SharedResourceHandle<T, E> {
        SharedResourceHandle { inner: self }
    }
}

impl ResourceHandle<dyn Any + Send + Sync, dyn std::error::Error + Send + Sync> {
    fn downcast<T: Send + Sync, E: std::error::Error + Send + Sync>(self) -> std::result::Result<ResourceHandle<T, E>, InvalidDowncast> {
        let from_type = self.type_name;
        let to_type = std::any::type_name::<T>();
        let from_err = self.error_name;
        let to_err = std::any::type_name::<E>();
        Ok::<_, InvalidDowncast>(ResourceHandle {
            data: Arc::downcast(self.data).map_err(|_| InvalidDowncast { from: from_type, to: to_type })?,
            status: Arc::downcast(self.status).map_err(|_| InvalidDowncast { from: from_err, to: to_err })?,
            type_name: to_type,
            error_name: from_type,
            _type: PhantomData,
            _error: PhantomData
        })
    }
}

pub struct SharedResourceHandle<T: 'static, E: std::error::Error> {
    inner: ResourceHandle<T, E>
}

impl<T, E: std::error::Error + Clone + 'static> SharedResourceHandle<T, E> {
    pub fn value(&self) -> Option<std::result::Result<&T, ResourceLoadError<E>>> {
        self.inner.value()
    }

    pub fn is_loaded(&self) -> bool {
        self.inner.is_loaded()
    }

    pub fn can_take(&self) -> bool {
        self.inner.can_take()
    }

    pub fn take(self) -> std::result::Result<Option<std::result::Result<T, ResourceLoadError<E>>>, Self> {
        self.inner.take().map_err(|err| err.share())
    }
}

impl<T: 'static, E: std::error::Error + Clone + 'static> From<ResourceHandle<T, E>> for SharedResourceHandle<T, E> {
    fn from(value: ResourceHandle<T, E>) -> Self {
        value.share()
    }
}

impl<T: 'static, E: std::error::Error + Clone> Clone for SharedResourceHandle<T, E> {
    fn clone(&self) -> Self {
        SharedResourceHandle { inner: clone_handle(&self.inner) }
    }
}

fn clone_handle<T, E: std::error::Error + Clone>(handle: &ResourceHandle<T, E>) -> ResourceHandle<T, E> {
    ResourceHandle { data: handle.data.clone(), status: handle.status.clone(), type_name: handle.type_name, error_name: handle.error_name, _type: handle._type, _error: handle._error }
}

struct ResourceHandleWeak<T: ?Sized, E: std::error::Error + ?Sized> {
    data: Weak<ResourceData>,
    status: Weak<RwLock<ResourceStatus>>,
    type_name: &'static str,
    error_name: &'static str,
    _type: PhantomData<T>,
    _error: PhantomData<E>
}

impl<T: ?Sized, E: std::error::Error + ?Sized> ResourceHandleWeak<T, E> {
    fn upgrade(&self) -> Option<ResourceHandle<T, E>> {
        let Self { data, status, type_name, error_name, _type, _error } = &self;
        let (type_name, error_name, _type, _error) = (*type_name, *error_name, *_type, *_error);

        let data = data.upgrade()?;
        let status = status.upgrade()?;

        Some(ResourceHandle { data, status, type_name, error_name, _type, _error })
    }

    fn into_any(self) -> ResourceHandleWeak<dyn Any + Send + Sync, dyn std::error::Error + Send + Sync> {
        let ResourceHandleWeak { data, status, type_name, error_name, .. } = self;

        ResourceHandleWeak { data, status, type_name, error_name, _type: PhantomData, _error: PhantomData }
    }
}

enum ResourceStatus {
    Unloaded,
    Loading,
    Loaded,
    Error
}

impl ResourceStatus {
    pub fn is_unloaded(&self) -> bool {
        matches!(self, Self::Unloaded)
    }
}

pub trait ResourceLoader<T, E: std::error::Error + Clone>: Send + Sync {
    fn load(self, data: Box<[u8]>) -> std::result::Result<T, E>;
}

type ResourceData = OnceLock<std::result::Result<Box<dyn Any + Send + Sync>, ResourceLoadError<dyn std::error::Error + Send + Sync>>>;

pub struct ResourceManager {
    asset_packs: HashMap<String, Arc<AssetPack>>,
    resources: HashMap<PathBuf, ResourceHandleWeak<dyn Any + Send + Sync, dyn std::error::Error + Send + Sync>>,
}

impl ResourceManager {
    pub(in crate::engine) fn new() -> ResourceManager {
        Self { asset_packs: HashMap::new(), resources: HashMap::new() }
    }

    pub fn load<T: Send + Sync +'static, E: std::error::Error + Send + Sync + 'static + Clone, L: ResourceLoader<T, E> + 'static, P: AsRef<Path>>(&mut self, loader: L, resource: P) -> Result<ResourceHandle<T, E>, ResourceError> {
        let handle = self.resources.get(resource.as_ref())
            .and_then(ResourceHandleWeak::upgrade)
            .map(ResourceHandle::downcast)
            .unwrap_or(Ok(ResourceHandle::<T, E>::new()))?;

        let loader_handle = clone_handle(&handle);
        // Give up instead of blocking because if the status is being writen, that means it is already being loaded
        if let Ok(status) = handle.status.try_read() {
            if status.is_unloaded() {
                drop(status);
                let handle = loader_handle;
                let (asset_pack, resource): (PathBuf, PathBuf) = resource.as_ref().components()
                    .enumerate()
                    .partition_map(|(i, component)| if i == 0 { itertools::Either::Left(component) } else { itertools::Either::Right(component) });
                
                let asset_pack_name = asset_pack.to_string_lossy().to_string();

                if !self.asset_packs.contains_key(&asset_pack_name) {
                    let path = asset_pack.with_added_extension("pack");
                    if !path.exists() {
                        Err(std::io::Error::from(NotFound))?;
                    }

                    let asset_pack = AssetPack::new(path).unwrap();
                    self.asset_packs.insert(asset_pack_name.clone(), asset_pack);
                }

                let asset_pack = self.asset_packs.get(&asset_pack_name).unwrap().clone();

                let before_load = {
                    let handle = clone_handle(&handle);
                    move || {
                        *handle.status.write().unwrap() = ResourceStatus::Loading;
                    }
                };

                let on_load = move |result: std::result::Result<Box<[u8]>, ResourcePackagerError>| {
                    let result = result
                        .map_err(Arc::new)
                        .map_err(Into::<ResourceLoadError<E>>::into)
                        .and_then(|data| {
                            let value: Box<dyn Any + Send + Sync> = Box::new(loader.load(data).map_err(|err| LoadError(Box::new(err)))?);
                            Ok(value)
                        })
                        .map_err(|err| err.into_any());
                    let status = match &result {
                        Ok(_) => ResourceStatus::Loaded,
                        Err(_) => ResourceStatus::Error,
                    };
                    handle.data.set(result).unwrap();
                    *handle.status.write().unwrap() = status;
                };
                asset_pack.load_resource(resource, before_load, on_load);
            }
        };

        if !self.resources.contains_key(resource.as_ref()) {
            self.resources.insert(resource.as_ref().to_path_buf(), handle.downgrade().into_any());
        }
        
        let handle = self.resources.get(resource.as_ref())
            .and_then(ResourceHandleWeak::upgrade)
            .map(ResourceHandle::downcast)
            .unwrap_or(Ok(ResourceHandle::<T, E>::new()))?;

        Ok(handle)
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

pub mod error {
    use std::{any::Any, sync::Arc};

    use downcast_rs::Downcast;
    use error::{Error, union};
    use resource_packager::packager::ResourcePackagerError;
    use crate::{error as errors_module};

    #[derive(Error, Debug, Clone, Copy)]
    #[error("Attempted to downcast type '{from}' to '{to}.")]
    pub struct InvalidDowncast {
        pub to: &'static str,
        pub from: &'static str
    }

    #[derive(Error, Debug, Clone)]
    #[error("{0}")]
    pub struct LoadError<E: std::error::Error + ?Sized>(pub Box<E>);

    union!(LoadError<E>: LoadError, Arc<ResourcePackagerError>: ResourcePackagerError, InvalidDowncast as #[derive(Clone)] ResourceLoadError<E: std::error::Error + ?Sized>);

    impl ResourceLoadError<dyn std::error::Error + Send + Sync> {
        pub fn downcast_and_clone<T: std::error::Error + Clone + 'static>(&self) -> Option<ResourceLoadError<T>> {
            Some(match self {
                ResourceLoadError::LoadError(load_error) => ResourceLoadError::LoadError({
                    let any: &dyn Any = load_error.0.as_any();
                    let downcast: &T = any.downcast_ref()?;
                    
                    LoadError(Box::new(downcast.clone()))
                }),
                ResourceLoadError::ResourcePackagerError(resource_packager_error) => ResourceLoadError::ResourcePackagerError(resource_packager_error.clone()),
                ResourceLoadError::InvalidDowncast(invalid_downcast) => ResourceLoadError::InvalidDowncast(*invalid_downcast),
            })
        }

        pub fn downcast<T: std::error::Error + 'static>(self) -> Option<ResourceLoadError<T>> {
            Some(match self {
                ResourceLoadError::LoadError(load_error) => ResourceLoadError::LoadError({
                    let any: Box<dyn std::error::Error> = load_error.0;
                    let downcast: Box<T> = any.downcast().ok()?;
                    
                    LoadError(downcast)
                }),
                ResourceLoadError::ResourcePackagerError(resource_packager_error) => ResourceLoadError::ResourcePackagerError(resource_packager_error),
                ResourceLoadError::InvalidDowncast(invalid_downcast) => ResourceLoadError::InvalidDowncast(invalid_downcast),
            })
        }
    }

    impl<E: std::error::Error + Send + Sync + 'static> ResourceLoadError<E> {
        pub(in crate::engine::resources::resource_manager) fn into_any(self) -> ResourceLoadError<dyn std::error::Error + Send + Sync> {
            match self {
                ResourceLoadError::LoadError(load_error) => ResourceLoadError::LoadError(LoadError(Box::new(load_error.0))),
                ResourceLoadError::ResourcePackagerError(resource_packager_error) => ResourceLoadError::ResourcePackagerError(resource_packager_error),
                ResourceLoadError::InvalidDowncast(invalid_downcast) => ResourceLoadError::InvalidDowncast(invalid_downcast),
            }
        }
    }

    union!(std::io::Error, InvalidDowncast as ResourceError);
}