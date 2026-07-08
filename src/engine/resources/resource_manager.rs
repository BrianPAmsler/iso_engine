use std::{any::Any, collections::{BTreeMap, HashMap}, io::ErrorKind::NotFound, marker::PhantomData, path::{Path, PathBuf}, sync::{Arc, OnceLock, RwLock, Weak}};

use itertools::Itertools;
use resource_packager::packager::{ResourcePackagerError, read::DirEntry};

use crate::{engine::resources::{error::{_ResourceLoadError, InvalidDowncast, ResourceError, ResourceLoadError}, pack::AssetPack}, error::Result};

#[derive(Debug)]
pub struct ResourceHandle<T: ?Sized + 'static, E: std::error::Error + ?Sized> {
    data: Arc<ResourceData>,
    status: Arc<RwLock<ResourceStatus>>,
    type_name: &'static str,
    error_name: &'static str,
    _type: PhantomData<T>,
    _error: PhantomData<E>
}

impl<T, E: std::error::Error + 'static> ResourceHandle<T, E> {
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
                        let err: Option<&_ResourceLoadError<E>> = err.downcast_ref();
                        println!("downcast ref successful");
                        let to = std::any::type_name::<E>();
                        let from = self.error_name;
                        match err {
                            Some(value) => value.as_outer(),
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
        #[allow(clippy::unwrap_used, reason="Strong count is already confirmed to be 1.")]
        let inner = Arc::into_inner(self.data).unwrap();
        let result = inner.into_inner()
            .map(|result| {
                result
                    .map_err(|err| {
                        let err: Option<Box<_ResourceLoadError<E>>> = err.downcast().ok();
                        let to = std::any::type_name::<E>();
                        let from = self.error_name;
                        match err {
                            Some(boxed) => boxed.into_outer(),
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

impl<T, E: std::error::Error + 'static> SharedResourceHandle<T, E> {
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

impl<T: 'static, E: std::error::Error + 'static> From<ResourceHandle<T, E>> for SharedResourceHandle<T, E> {
    fn from(value: ResourceHandle<T, E>) -> Self {
        value.share()
    }
}

impl<T: 'static, E: std::error::Error> Clone for SharedResourceHandle<T, E> {
    fn clone(&self) -> Self {
        SharedResourceHandle { inner: clone_handle(&self.inner) }
    }
}

fn clone_handle<T, E: std::error::Error>(handle: &ResourceHandle<T, E>) -> ResourceHandle<T, E> {
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

#[derive(Debug)]
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

pub trait ResourceLoader<T, E: std::error::Error>: Send + Sync {
    fn load(self, data: Box<[u8]>) -> std::result::Result<T, E>;
}

pub trait MultiResourceLoader<T, E: std::error::Error>: Send + Sync {
    fn load(self, data: BTreeMap<PathBuf, Box<[u8]>>) -> std::result::Result<T, E>;
}

type ResourceData = OnceLock<std::result::Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>>;

pub struct ResourceManager {
    asset_packs: HashMap<String, Arc<AssetPack>>,
    resources: HashMap<PathBuf, ResourceHandleWeak<dyn Any + Send + Sync, dyn std::error::Error + Send + Sync>>,
}

impl ResourceManager {
    pub(in crate::engine) fn new() -> ResourceManager {
        Self { asset_packs: HashMap::new(), resources: HashMap::new() }
    }

    pub fn read_dir<P: AsRef<Path>>(&self, resource_dir: P) -> Vec<DirEntry> {
        let (asset_pack_path, resource): (PathBuf, PathBuf) = resource_dir.as_ref().components()
            .enumerate()
            .partition_map(|(i, component)| if i == 0 { itertools::Either::Left(component) } else { itertools::Either::Right(component) });

        let asset_pack_string = asset_pack_path.to_string_lossy().into_owned();

        let Some(asset_pack) = self.asset_packs.get(&asset_pack_string) else { return Vec::new() };
        asset_pack.read_dir(resource).into_iter().map(|mut entry| {
            match &mut entry {
                DirEntry::File(path_buf) | DirEntry::Directory(path_buf)=> {
                    let new_path = asset_pack_path.join(&path_buf);
                    *path_buf = new_path;
                }
            }
            entry
        }).collect_vec()
    }

    pub fn load_file<T: Send + Sync +'static, E: std::error::Error + Send + Sync + 'static, L: ResourceLoader<T, E> + 'static, P: AsRef<Path>>(&mut self, loader: L, resource: P) -> Result<ResourceHandle<T, E>, ResourceError> {
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
                
                let asset_pack_name = asset_pack.to_string_lossy().into_owned();

                if !self.asset_packs.contains_key(&asset_pack_name) {
                    let path = asset_pack.with_added_extension("pack");
                    if !path.exists() {
                        Err(std::io::Error::from(NotFound))?;
                    }

                    let asset_pack = AssetPack::new(path).unwrap();
                    self.asset_packs.insert(asset_pack_name.clone(), asset_pack);
                }

                #[allow(clippy::unwrap_used, reason="Key already checked.")]
                let asset_pack = self.asset_packs.get(&asset_pack_name).unwrap().clone();

                let before_load = {
                    let handle = clone_handle(&handle);
                    #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
                    move || {
                        *handle.status.write().unwrap() = ResourceStatus::Loading;
                    }
                };

                let on_load = move |result: std::result::Result<Box<[u8]>, ResourcePackagerError>| {
                    let result = result
                        .map_err(Arc::new)
                        .map_err(_ResourceLoadError::ResourcePackagerError)
                        .and_then(|data| {
                            let value: Box<dyn Any + Send + Sync> = Box::new(loader.load(data).map_err(|err| _ResourceLoadError::LoadError(Arc::new(err)))?);
                            Ok(value)
                        })
                        .map_err(Box::new)
                        .map_err(|err| err as Box<dyn Any + Send + Sync>);
                    let status = match &result {
                        Ok(_) => ResourceStatus::Loaded,
                        Err(_) => ResourceStatus::Error,
                    };
                    #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
                    handle.data.set(result).unwrap();
                    #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
                    {*handle.status.write().unwrap() = status;}
                };
                asset_pack.load_resource(resource, before_load, on_load);
            }
        };

        if !self.resources.contains_key(resource.as_ref()) {
            self.resources.insert(resource.as_ref().to_path_buf(), handle.downgrade().into_any());
        }

        Ok(handle)
    }

    pub fn load_dir<T: Send + Sync +'static, E: std::error::Error + Send + Sync + 'static, L: MultiResourceLoader<T, E> + 'static, P: AsRef<Path>>(&mut self, loader: L, dir: P) -> Result<ResourceHandle<T, E>, ResourceError> {
        let handle = self.resources.get(dir.as_ref())
            .and_then(ResourceHandleWeak::upgrade)
            .map(ResourceHandle::downcast)
            .unwrap_or(Ok(ResourceHandle::<T, E>::new()))?;

        let loader_handle = clone_handle(&handle);
        // Give up instead of blocking because if the status is being writen, that means it is already being loaded
        if let Ok(status) = handle.status.try_read() {
            if status.is_unloaded() {
                drop(status);
                let handle = loader_handle;
                let (asset_pack, dir): (PathBuf, PathBuf) = dir.as_ref().components()
                    .enumerate()
                    .partition_map(|(i, component)| if i == 0 { itertools::Either::Left(component) } else { itertools::Either::Right(component) });
                
                let asset_pack_name = asset_pack.to_string_lossy().into_owned();

                if !self.asset_packs.contains_key(&asset_pack_name) {
                    let path = asset_pack.with_added_extension("pack");
                    if !path.exists() {
                        Err(std::io::Error::from(NotFound))?;
                    }

                    let asset_pack = AssetPack::new(path).unwrap();
                    self.asset_packs.insert(asset_pack_name.clone(), asset_pack);
                }

                #[allow(clippy::unwrap_used, reason="Key already checked.")]
                let asset_pack = self.asset_packs.get(&asset_pack_name).unwrap().clone();

                let before_load = {
                    let handle = clone_handle(&handle);
                    #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
                    move || {
                        *handle.status.write().unwrap() = ResourceStatus::Loading;
                    }
                };

                let on_load = move |result:std::result::Result<BTreeMap<PathBuf, Box<[u8]>>, ResourcePackagerError>| {
                    let result = result
                        .map_err(Arc::new)
                        .map_err(_ResourceLoadError::ResourcePackagerError)
                        .and_then(|data| {
                            let value: Box<dyn Any + Send + Sync> = Box::new(loader.load(data).map_err(|err| _ResourceLoadError::LoadError(Arc::new(err)))?);
                            Ok(value)
                        })
                        .map_err(Box::new)
                        .map_err(|err| err as Box<dyn Any + Send + Sync>);
                    let status = match &result {
                        Ok(_) => ResourceStatus::Loaded,
                        Err(_) => ResourceStatus::Error,
                    };
                    #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
                    handle.data.set(result).unwrap();
                    #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
                    {*handle.status.write().unwrap() = status;}
                };
                asset_pack.load_dir(dir, before_load, on_load);
            }
        };

        if !self.resources.contains_key(dir.as_ref()) {
            self.resources.insert(dir.as_ref().to_path_buf(), handle.downgrade().into_any());
        }

        Ok(handle)
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

pub mod error {
    use std::sync::Arc;

    use error::{Error, union};
    use resource_packager::packager::ResourcePackagerError;
    use crate::{error as errors_module};

    #[derive(Error, Debug, Clone, Copy)]
    #[error("Attempted to downcast type '{from}' to '{to}'.")]
    pub struct InvalidDowncast {
        pub to: &'static str,
        pub from: &'static str
    }

    #[derive(Error, Debug, Clone)]
    #[error("{0}")]
    pub struct LoadError<E: std::error::Error>(pub Arc<E>);

    union!(LoadError<E>: LoadError, Arc<ResourcePackagerError>: ResourcePackagerError, InvalidDowncast as ResourceLoadError<E: std::error::Error>);

    pub(in crate::engine::resources::resource_manager) enum _ResourceLoadError<E> {        
        LoadError(Arc<E>),
        ResourcePackagerError(Arc<ResourcePackagerError>)
    }

    impl<E: std::error::Error> _ResourceLoadError<E> {
        pub fn into_outer(self) -> ResourceLoadError<E> {
            match self {
                _ResourceLoadError::LoadError(err) => ResourceLoadError::LoadError(LoadError(err)),
                _ResourceLoadError::ResourcePackagerError(resource_packager_error) => ResourceLoadError::ResourcePackagerError(resource_packager_error)
            }
        }

        pub fn as_outer(&self) -> ResourceLoadError<E> {
            match self {
                _ResourceLoadError::LoadError(err) => ResourceLoadError::LoadError(LoadError(err.clone())),
                _ResourceLoadError::ResourcePackagerError(resource_packager_error) => ResourceLoadError::ResourcePackagerError(resource_packager_error.clone())
            }
        }
    }

    union!(std::io::Error, InvalidDowncast as ResourceError);
}