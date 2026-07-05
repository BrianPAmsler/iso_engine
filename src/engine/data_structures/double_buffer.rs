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
        let read_buffer = if *self.unswapped.lock().unwrap() {
            &self.a
        } else {
            &self.b
        };

        ReadGuard(read_buffer.read().unwrap())
    }

    pub fn write(&self) -> WriteGuard<'_, T> {
        let write_buffer = if *self.unswapped.lock().unwrap() {
            &self.b
        } else {
            &self.a
        };

        WriteGuard(write_buffer.write().unwrap())
    }

    pub fn swap(&self) {
        let (read_buffer, write_buffer) = if *self.unswapped.lock().unwrap() {
            (&self.a, &self.b)
        } else {
            (&self.b, &self.a)
        };

        // Acquire both buffers
        let _write_buffer = write_buffer.read().unwrap();
        let mut read_buffer = read_buffer.write().unwrap();

        read_buffer.clear();
        let mut unswapped = self.unswapped.lock().unwrap();
        *unswapped = !*unswapped;
    }
}

impl<T: Clone> DoubleBuffer<T> {
    pub fn clone_and_swap(&self) {
        let (read_buffer, write_buffer) = if *self.unswapped.lock().unwrap() {
            (&self.a, &self.b)
        } else {
            (&self.b, &self.a)
        };

        // Acquire both buffers
        let write_buffer = write_buffer.read().unwrap();
        let mut read_buffer = read_buffer.write().unwrap();

        read_buffer.clone_from(&write_buffer);
        let mut unswapped = self.unswapped.lock().unwrap();
        *unswapped = !*unswapped;
    }
}

impl<T> Default for DoubleBuffer<T> {
    fn default() -> Self {
        DoubleBuffer::new()
    }
}