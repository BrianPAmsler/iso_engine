use std::{any::Any, collections::HashMap, io::ErrorKind::NotFound, marker::PhantomData, path::{Path, PathBuf}, sync::{Arc, RwLock, Weak}};

use itertools::Itertools;
use resource_packager::packager::ResourcePackagerError;

use crate::{engine::resources::pack::AssetPack, error::{ExplicitUnwrap, Result}};

#[derive(Clone)]
pub struct ResourceHandle<T> {
    data: Arc<RwLock<ResourceData>>,
    _pd: PhantomData<T>
}

impl<T> ResourceHandle<T> {
    fn new() -> ResourceHandle<T>  {
        ResourceHandle {
            data: Arc::new(RwLock::new(ResourceStatus::Unloaded)),
            _pd: PhantomData,
        }
    }
}

enum ResourceStatus<T, E> {
    Unloaded,
    Loading,
    Loaded(T),
    Error(E)
}

impl<T, E> ResourceStatus<T, E> {
    pub fn is_unloaded(&self) -> bool {
        matches!(self, Self::Unloaded)
    }
}

pub trait ResourceLoader<T>: Send + Sync {
    fn load(self, data: Box<[u8]>) -> T;
}

type ResourceData = ResourceStatus<Box<dyn Any + Send + Sync>, ResourcePackagerError>;

pub struct ResourceManager {
    asset_packs: HashMap<String, Arc<AssetPack>>,
    resources: HashMap<PathBuf, Weak<RwLock<ResourceData>>>,
}

impl ResourceManager {
    fn load<T: Send + Sync + 'static, L: ResourceLoader<T> + 'static, P: AsRef<Path>>(&mut self, loader: L, resource: P) -> Result<ResourceHandle<T>, std::io::Error> {
        let data = self.resources.get(resource.as_ref())
            .and_then(|weak| weak.upgrade())
            .unwrap_or(Arc::new(RwLock::new(ResourceStatus::Unloaded)));

        let loader_data = data.clone();
        if let Ok(read_lock) = data.try_read() {
            if read_lock.is_unloaded() {
                let data = loader_data;
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

                drop(read_lock);
                *data.write().explicit_unwrap() = ResourceStatus::Loading;

                let asset_pack = self.asset_packs.get(&asset_pack_name).explicit_unwrap().clone();
                asset_pack.load_resource(resource, move |result| {
                    let result = result.map(|data| {
                        let value: Box<dyn Any + Send + Sync> = Box::new(loader.load(data));
                        value
                    });
                    let status = match result {
                        Ok(value) => ResourceStatus::Loaded(value),
                        Err(err) => ResourceStatus::Error(err),
                    };
                    *data.write().explicit_unwrap() = status;
                });
            }
        };

        Ok(ResourceHandle { data, _pd: PhantomData })
    }
}