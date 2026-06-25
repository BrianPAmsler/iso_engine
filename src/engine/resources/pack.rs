use std::{collections::VecDeque, fs::File, path::{Path, PathBuf}, sync::{Arc, Mutex, atomic::AtomicBool}, time::Duration};

use resource_packager::packager::{ResourcePackagerError, read::ResourcePackageReader};

use crate::error::{ExplicitUnwrap, Result};

trait Callback: FnOnce(std::result::Result<Box<[u8]>, ResourcePackagerError>) + Send + Sync {}
impl<F: FnOnce(std::result::Result<Box<[u8]>, ResourcePackagerError>) + Send + Sync> Callback for F {}

struct LoadCommand {
    path: PathBuf,
    callback: Box<dyn Callback>
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

                let Some(command) = queue.pop_front() else { continue };
                drop(queue);

                let Ok(mut reader) = pack.reader.try_lock() else { continue };

                let result = reader.read_file(command.path);
                drop(reader);

                (command.callback)(result);
            }
        });
    }

    pub fn load_resource<P, F>(&self, path: P, callback: F)
    where
        P: Into<PathBuf>,
        F: FnOnce(std::result::Result<Box<[u8]>, ResourcePackagerError>) + Send + Sync + 'static
    {
        let path = path.into();
        let callback = Box::new(callback);
        let mut queue = self.load_queue.lock().explicit_unwrap();

        queue.push_back(LoadCommand { path, callback });
    }
}