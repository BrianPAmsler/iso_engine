use std::{ops::{Deref, DerefMut}, sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard}};

pub struct ReadGuard<'a, T>(RwLockReadGuard<'a, Vec<T>>);

impl<'a, T> Deref for ReadGuard<'a, T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub struct WriteGuard<'a, T>(RwLockWriteGuard<'a, Vec<T>>);

impl<'a, T> Deref for WriteGuard<'a, T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'a, T> DerefMut for WriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

pub struct DoubleBuffer<T> {
    unswapped: Mutex<bool>,
    a: RwLock<Vec<T>>,
    b: RwLock<Vec<T>>
}

impl<T> DoubleBuffer<T> {
    pub fn new() -> DoubleBuffer<T> {
        DoubleBuffer { unswapped: Mutex::new(true), a: RwLock::new(Vec::new()), b: RwLock::new(Vec::new()) }
    }

    pub fn read(&self) -> ReadGuard<'_, T> {
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let read_buffer = if *self.unswapped.lock().unwrap() {
            &self.a
        } else {
            &self.b
        };

        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        ReadGuard(read_buffer.read().unwrap())
    }

    pub fn write(&self) -> WriteGuard<'_, T> {
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let write_buffer = if *self.unswapped.lock().unwrap() {
            &self.b
        } else {
            &self.a
        };

        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        WriteGuard(write_buffer.write().unwrap())
    }

    pub fn swap(&self) {
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let (read_buffer, write_buffer) = if *self.unswapped.lock().unwrap() {
            (&self.a, &self.b)
        } else {
            (&self.b, &self.a)
        };

        // Acquire both buffers
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let _write_buffer = write_buffer.read().unwrap();
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let mut read_buffer = read_buffer.write().unwrap();

        read_buffer.clear();
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let mut unswapped = self.unswapped.lock().unwrap();
        *unswapped = !*unswapped;
    }
}

impl<T: Clone> DoubleBuffer<T> {
    pub fn clone_and_swap(&self) {
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let (read_buffer, write_buffer) = if *self.unswapped.lock().unwrap() {
            (&self.a, &self.b)
        } else {
            (&self.b, &self.a)
        };

        // Acquire both buffers
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let write_buffer = write_buffer.read().unwrap();
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let mut read_buffer = read_buffer.write().unwrap();

        read_buffer.clone_from(&write_buffer);
        #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
        let mut unswapped = self.unswapped.lock().unwrap();
        *unswapped = !*unswapped;
    }
}

impl<T> Default for DoubleBuffer<T> {
    fn default() -> Self {
        DoubleBuffer::new()
    }
}