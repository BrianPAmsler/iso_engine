use std::{collections::{BTreeMap, VecDeque}, fs::File, io::BufReader, path::{Path, PathBuf}, sync::{Arc, Mutex, atomic::AtomicBool}, time::Duration};

use itertools::Itertools;
use resource_packager::packager::{ResourcePackagerError, read::{DirEntry, ResourcePackageReader}};

use crate::error::Result;

trait OnLoadFile: FnOnce(std::result::Result<Box<[u8]>, ResourcePackagerError>) + Send + Sync {}
impl<F: FnOnce(std::result::Result<Box<[u8]>, ResourcePackagerError>) + Send + Sync> OnLoadFile for F {}

trait OnLoadDir: FnOnce(std::result::Result<BTreeMap<PathBuf, Box<[u8]>>, ResourcePackagerError>) + Send + Sync {}
impl<F: FnOnce(std::result::Result<BTreeMap<PathBuf, Box<[u8]>>, ResourcePackagerError>) + Send + Sync> OnLoadDir for F {}

trait BeforeLoad: FnOnce() + Send + Sync {}
impl<F: FnOnce() + Send + Sync> BeforeLoad for F {}

enum Command {
    LoadFile {
        path: PathBuf,
        before_load: Box<dyn BeforeLoad>,
        on_load: Box<dyn OnLoadFile>
    },
    LoadDir {
        path: PathBuf,
        before_load: Box<dyn BeforeLoad>,
        on_load: Box<dyn OnLoadDir>
    }
}

pub struct AssetPack {
    reader: Mutex<ResourcePackageReader<BufReader<File>>>,
    load_queue: Mutex<VecDeque<Command>>,
    open: AtomicBool
}

impl AssetPack {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Arc<AssetPack>, ResourcePackagerError> {
        let file = BufReader::new(File::open(path).map_err(ResourcePackagerError::IoError)?);

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

                let Some(command) = queue.pop_front() else { continue };
                drop(queue);

                match command {
                    Command::LoadFile { path, before_load, on_load } => {
                        before_load();
                        let Ok(mut reader) = pack.reader.try_lock() else { continue };

                        let result = reader.read_file(path);
                        drop(reader);

                        on_load(result);
                    },
                    Command::LoadDir { path, before_load, on_load } => {
                        before_load();
                        let Ok(mut reader) = pack.reader.try_lock() else { continue };

                        let result = (|| {
                            reader.read_dir(path)?.into_iter()
                                .filter_map(|entry| match entry {
                                    DirEntry::File(file) => Some(file),
                                    _ => None
                                })
                                .map(|file| {
                                    let data = reader.read_file(&file)?;
                                    Ok((file, data))
                                })
                                .try_collect()
                        })();
                        drop(reader);

                        on_load(result);
                    },
                }
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

        queue.push_back(Command::LoadFile { path, before_load, on_load });
    }

    pub fn load_dir<P, F1, F2>(&self, path: P, before_load: F1, on_load: F2)
    where
        P: Into<PathBuf>,
        F1: FnOnce() + Send + Sync + 'static,
        F2: FnOnce(std::result::Result<BTreeMap<PathBuf, Box<[u8]>>, ResourcePackagerError>) + Send + Sync + 'static
    {
        let path = path.into();
        let before_load = Box::new(before_load);
        let on_load = Box::new(on_load);
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let mut queue = self.load_queue.lock().unwrap();

        queue.push_back(Command::LoadDir { path, before_load, on_load });
    }

    pub fn read_dir<P: AsRef<Path>>(&self, resource_dir: P) -> Vec<DirEntry> {
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let reader = self.reader.lock().unwrap();

        reader.read_dir(resource_dir).unwrap_or_default()
    }
}