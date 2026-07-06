use std::{collections::VecDeque, fs::File, path::{Path, PathBuf}, sync::{Arc, Mutex, atomic::AtomicBool}, time::Duration};

use resource_packager::packager::{ResourcePackagerError, read::{DirEntry, ResourcePackageReader}};

use crate::error::Result;

trait OnLoad: FnOnce(std::result::Result<Box<[u8]>, ResourcePackagerError>) + Send + Sync {}
impl<F: FnOnce(std::result::Result<Box<[u8]>, ResourcePackagerError>) + Send + Sync> OnLoad for F {}

trait BeforeLoad: FnOnce() + Send + Sync {}
impl<F: FnOnce() + Send + Sync> BeforeLoad for F {}

struct LoadCommand {
    path: PathBuf,
    before_load: Box<dyn BeforeLoad>,
    on_load: Box<dyn OnLoad>
}

pub struct AssetPack {
    reader: Mutex<ResourcePackageReader<File>>,
    load_queue: Mutex<VecDeque<LoadCommand>>,
    open: AtomicBool
}

impl AssetPack {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Arc<AssetPack>, ResourcePackagerError> {
        let file = File::open(path).map_err(ResourcePackagerError::IoError)?;

        let todo = (); // TODO: Move this into the io thread
        let reader = Mutex::new(ResourcePackageReader::new(file)?);

        let pack = Arc::new(AssetPack {
            reader,
            load_queue: Mutex::new(VecDeque::new()),
            open: AtomicBool::new(true),
        });

        Self::run(pack.clone());

        Ok(pack)
    }

    fn run(pack: Arc<Self>) {
        std::thread::spawn(move || {
            while pack.open.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(10));
                let Ok(mut queue) = pack.load_queue.try_lock() else { continue };

                let Some(LoadCommand { path, before_load, on_load }) = queue.pop_front() else { continue };
                drop(queue);

                before_load();
                let Ok(mut reader) = pack.reader.try_lock() else { continue };

                let result = reader.read_file(path);
                drop(reader);

                on_load(result);
            }
        });
    }

    pub fn load_resource<P, F1, F2>(&self, path: P, before_load: F1, on_load: F2)
    where
        P: Into<PathBuf>,
        F1: FnOnce() + Send + Sync + 'static,
        F2: FnOnce(std::result::Result<Box<[u8]>, ResourcePackagerError>) + Send + Sync + 'static
    {
        let path = path.into();
        let before_load = Box::new(before_load);
        let on_load = Box::new(on_load);
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let mut queue = self.load_queue.lock().unwrap();

        queue.push_back(LoadCommand { path, before_load, on_load });
    }

    pub fn read_dir<P: AsRef<Path>>(&self, resource_dir: P) -> Vec<DirEntry> {
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let reader = self.reader.lock().unwrap();

        reader.read_dir(resource_dir).unwrap_or_default()
    }
}