use std::{collections::VecDeque, fmt::Debug};

use rand::RngExt as _;

use super::error::Result;

#[derive(Debug, Clone)]
enum Slot<T> {
    Element { id: usize, value: T },
    Hole { id: usize, next: usize }
}

pub struct VecAllocator<T> {
    id: u128,
    vec: Vec<Slot<T>>,
    first_hole: usize,
    count: usize
}

impl<T: Debug> Debug for VecAllocator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecAllocator").field("vec", &self.vec).field("first_hole", &self.first_hole).field("count", &self.count).finish()
    }
}

impl<T: Clone> Clone for VecAllocator<T> {
    fn clone(&self) -> Self {
        Self {
            id: generate_id(),
            vec: self.vec.clone(),
            first_hole: self.first_hole.clone(),
            count: self.count.clone()
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct AllocationIndex {
    allocator_id: u128,
    index: usize,
    id: usize
}

impl AllocationIndex {
    pub fn null() -> AllocationIndex {
        AllocationIndex { allocator_id: 0, index: 0, id: 0 }
    }

    pub fn ptr_eq<T>(&self, allocator: &VecAllocator<T>) -> bool {
        self.allocator_id == allocator.id
    }
}

fn generate_id() -> u128 {
    rand::rng().random::<u128>().saturating_add(1)
}

impl<T: Clone> VecAllocator<T> {
    #[cfg(test)]
    fn from_raw(slice: &[Slot<T>]) -> VecAllocator<T> {
        let mut vec = Vec::new();
        vec.extend_from_slice(slice);

        let mut first_hole = vec.len();
        let mut count = 0;
        for (i, slot) in vec.iter().enumerate() {
            match slot {
                Slot::Hole { .. } => {
                    if first_hole >= vec.len() {
                        first_hole = i;
                    }
                },
                Slot::Element { .. } => count += 1
            }
        }

        if first_hole == vec.len() {
            panic!("No hole!");
        }

        VecAllocator { id: generate_id(), first_hole, vec, count }
    }
}

impl<T> VecAllocator<T> {
    pub fn new() -> VecAllocator<T> {
        let mut vec = Vec::new();
        vec.push(Slot::Hole { id: 0, next: 0 });
        VecAllocator { id: generate_id(), vec, first_hole: 0, count: 0 }
    }

    pub fn capacity(&self) -> usize {
        self.vec.len()
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn insert(&mut self, value: T) -> AllocationIndex {
        match self.vec[self.first_hole] {
            Slot::Element { .. } => panic!("first_hole is invalid! This should be impossible, there is a bug somewhere!"),
            Slot::Hole { id, next } => {
                let new_slot = Slot::Element { id, value };

                // If there is only one hole, add a new hole at the end
                let mut next = next;
                if next <= self.first_hole {
                    next = self.vec.len();
                    let new_hole = Slot::Hole { id: 0, next };
                    self.vec.push(new_hole);
                }

                // Place the new element
                let index = AllocationIndex { allocator_id: self.id, index: self.first_hole, id };
                self.vec[self.first_hole] = new_slot;

                self.first_hole = next;
                self.count += 1;

                index
            }
        }
    }

    pub fn remove(&mut self, element: AllocationIndex) -> Result<T> {
        if !element.ptr_eq(self) {
            return Err(super::error::Error::IndexPointerMismatchError);
        }

        let id = match self.vec[element.index] {
            Slot::Hole { .. } => return Err(super::error::Error::ElementRemovedError),
            Slot::Element { id, value: _ } => id
        };

        if id != element.id {
            return Err(super::error::Error::ElementRemovedError);
        }

        let mut next = self.first_hole;
        let mut prev = None;

        while next < element.index {
            match &self.vec[next] {
                Slot::Element { .. } => panic!("something is fucked with VecAllocator!"),
                Slot::Hole { id: _, next: next_hole } => {
                    prev = Some(next);

                    if *next_hole == next {
                        break;
                    }

                    next = *next_hole;
                }
            }
        }

        match prev {
            Some(prev) => {
                match &mut self.vec[prev] {
                    Slot::Element { .. } => panic!("something is fucked with VecAllocator!"),
                    Slot::Hole { id: _, next } => {
                        *next = element.index;
                    }
                }
            },
            None => ()
        }

        let new_hole = Slot::Hole { id: id + 1, next };
        let old = std::mem::replace(&mut self.vec[element.index], new_hole);

        let old = match old {
            Slot::Element { value, .. } => value,
            Slot::Hole { .. } => panic!("this shouldn't be possible."),
        };

        if element.index < self.first_hole {
            self.first_hole = element.index;
        }

        self.count -= 1;

        Ok(old)
    }

    pub fn get(&self, element: AllocationIndex) -> Result<&T> {
        if !element.ptr_eq(self) {
            return Err(super::error::Error::IndexPointerMismatchError);
        }

        match &self.vec[element.index] {
            Slot::Element { id, value } => {
                if *id != element.id {
                    Err(super::error::Error::ElementRemovedError)
                } else {
                    Ok(value)
                }
            },
            Slot::Hole { .. } => Err(super::error::Error::ElementRemovedError),
        }
    }

    pub fn get_mut(&mut self, element: AllocationIndex) -> Result<&mut T> {
        if !element.ptr_eq(self) {
            return Err(super::error::Error::IndexPointerMismatchError);
        }
        
        match &mut self.vec[element.index] {
            Slot::Element { id, value } => {
                if *id != element.id {
                    Err(super::error::Error::ElementRemovedError)
                } else {
                    Ok(value)
                }
            },
            Slot::Hole { .. } => Err(super::error::Error::ElementRemovedError),
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter { allocator: self, index: 0 }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
        IterMut { slice: &mut self.vec, allocator_id: self.id, index: 0  }
    }
}

pub struct Iter<'a, T> {
    allocator: &'a VecAllocator<T>,
    index: usize
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (AllocationIndex, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.allocator.vec.len() {
            match &self.allocator.vec[self.index] {
                Slot::Element { id, value, .. } => {
                    let index = AllocationIndex { allocator_id: self.allocator.id, index: self.index, id: *id };
                    self.index += 1;
                    return Some((index, &value));
                },
                Slot::Hole { .. } => self.index += 1,
            }
        }

        None
    }
}

impl<'a, T> IntoIterator for &'a VecAllocator<T> {
    type Item = (AllocationIndex, &'a T);

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IterMut<'a, T> {
    slice: &'a mut [Slot<T>],
    allocator_id: u128,
    index: usize
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (AllocationIndex, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let mut slice = std::mem::take(&mut self.slice);
        let (mut first, mut rest) = slice.split_first_mut()?;

        while !rest.is_empty() {
            match first {
                Slot::Element { id, value } => {
                    self.slice = rest;
                    let index = AllocationIndex { allocator_id: self.allocator_id, index: self.index, id: *id };
                    self.index += 1;
                    return Some((index, value))
                },
                Slot::Hole { .. } => {
                    slice = rest;
                    (first, rest) = slice.split_first_mut()?;
                    self.index += 1;
                },
            }
        }

        None
    }
}

impl<'a, T> IntoIterator for &'a mut VecAllocator<T> {
    type Item = (AllocationIndex, &'a mut T);

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct IntoIter<T> {
    vec: VecDeque<Slot<T>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.vec.is_empty() {
            match self.vec.pop_front().unwrap() {
                Slot::Element { value, .. } => {
                    return Some(value);
                },
                Slot::Hole { .. } => ()
            }
        }

        None
    }
}

impl<T> IntoIterator for VecAllocator<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            vec: self.vec.into(),
        }
    }
}

#[cfg(test)]
mod tests {

    use rand::{RngExt};

    use super::{Slot, VecAllocator};

    fn insert<T: std::fmt::Debug>(vec: &mut Vec<Slot<T>>, value: T) {
        let mut slot = None;
        let mut i = 0;
        while slot.is_none() && i < vec.len() {
            let next_slot = &vec[i];

            match next_slot {
                Slot::Hole { id, next } => slot = Some((i, *id, *next)),
                _ => ()
            }

            i += 1;
        }

        match slot {
            Some((i, id, next)) => {
                vec[i] = Slot::Element { id, value };

                // Add hole at end if this was the last hole
                if next <= i {
                    vec.push(Slot::Hole { id: 0, next: vec.len() });
                }
            },
            None => panic!("Malformed vec!")
        }
    }

    fn remove<T: std::fmt::Debug>(vec: &mut Vec<Slot<T>>, index: usize) {
        let id = match &vec[index] {
            Slot::Element { id, .. } => *id + 1,
            _ => panic!("already hole!")
        };

        let new_hole = Slot::Hole { id, next: 0 };

        vec[index] = new_hole;

        // Update all holes
        let mut next_hole = vec.len() - 1;
        for (i, slot) in vec.iter_mut().enumerate().rev() {
            match slot {
                Slot::Element { .. } => (),
                Slot::Hole { next , .. } => {
                    *next = next_hole;
                    next_hole = i;
                },
            }
        }
    }

    fn test_iter<T: std::fmt::Debug + Clone + Copy + Eq>(mut allocator: VecAllocator<T>, list: &[Slot<T>]) -> Result<(), String> {
        let mut expected = Vec::new();
        for slot in list {
            match slot {
                Slot::Element { value, .. } => expected.push(*value),
                _ => ()
            }
        }

        let mut test = Vec::new();
        for (_, value) in &allocator {
            test.push(*value);
        }

        let mut test_mut = Vec::new();
        for (_, value) in &mut allocator {
            test_mut.push(*value);
        }

        let mut test_consuming = Vec::new();
        for value in allocator {
            test_consuming.push(value);
        }

        if expected != test {
            eprintln!("expected: {:?}", expected);
            eprintln!("actual: {:?}", test);

            return Err("VecAllocator.iter() failed!".into());
        }

        if expected != test_mut {
            eprintln!("expected: {:?}", expected);
            eprintln!("actual: {:?}", test_mut);

            return Err("VecAllocater.iter_mut() failed!".into());
        }

        if expected != test_consuming {
            eprintln!("expected: {:?}", expected);
            eprintln!("actual: {:?}", test_consuming);

            return Err("VecAllocater.iter_mut() failed!".into());
        }

        Ok(())
    }

    fn compare_vecs<T: std::fmt::Debug>(a: &VecAllocator<T>, b: &VecAllocator<T>) -> Result<(), String> {
        let string_a = format!("{:?}", *a);
        let string_b = format!("{:?}", *b);

        if string_a != string_b {
            eprintln!("A: {}", string_a);
            eprintln!("B: {}", string_b);
            return Err("Not equal!".into());
        }

        Ok(())
    }

    #[test]
    pub fn vec_allocator() -> Result<(), String> {
        let mut entries = Vec::new();
        let mut allocator = VecAllocator::new();
        let mut test_vec = Vec::new();

        let ten_elements = [
            Slot::Element { id: 0, value: 0 },
            Slot::Element { id: 0, value: 1 },
            Slot::Element { id: 0, value: 2 },
            Slot::Element { id: 0, value: 3 },
            Slot::Element { id: 0, value: 4 },
            Slot::Element { id: 0, value: 5 },
            Slot::Element { id: 0, value: 6 },
            Slot::Element { id: 0, value: 7 },
            Slot::Element { id: 0, value: 8 },
            Slot::Element { id: 0, value: 9 },
            Slot::Hole { id: 0, next: 10 }
        ];

        let expected = VecAllocator::from_raw(&ten_elements);
        test_vec.extend(ten_elements.into_iter());

        (0..10).for_each(|n| {
            let entry = allocator.insert(n);
            entries.push(entry);
        });

        compare_vecs(&expected, &allocator)?;

        for _ in 0..10000 {
            let mut rng = rand::rng();
            let choice: f32 = rng.random();

            if entries.len() == 0 || choice > 0.45 {
                let value: u128 = rng.random();

                println!("Inserting value {}...", value);
                insert(&mut test_vec, value);
                entries.push(allocator.insert(value));

                let expected = VecAllocator::from_raw(&test_vec);
                compare_vecs(&expected, &allocator)?;
                test_iter(allocator.clone(), &test_vec)?;
            } else {
                let index = rng.random_range(0..entries.len());
                let entry = entries.remove(index);

                println!("Removing at index {}...", entry.index);
                remove(&mut test_vec, entry.index);
                allocator.remove(entry).unwrap();

                let expected = VecAllocator::from_raw(&test_vec);
                compare_vecs(&expected, &allocator)?;
                test_iter(allocator.clone(), &test_vec)?;
            }
        }
        
        for _ in 0..1000 {
            let mut rng = rand::rng();
            let choice: f32 = rng.random();

            if entries.len() == 0 || choice > 0.9 {
                let value: u128 = rng.random();

                println!("Inserting value {}...", value);
                insert(&mut test_vec, value);
                entries.push(allocator.insert(value));

                let expected = VecAllocator::from_raw(&test_vec);
                compare_vecs(&expected, &allocator)?;
                test_iter(allocator.clone(), &test_vec)?;
            } else {
                let index = rng.random_range(0..entries.len());
                let entry = entries.remove(index);

                println!("Removing at index {}...", entry.index);
                remove(&mut test_vec, entry.index);
                allocator.remove(entry).unwrap();

                let expected = VecAllocator::from_raw(&test_vec);
                compare_vecs(&expected, &allocator)?;
                test_iter(allocator.clone(), &test_vec)?;
            }
        }

        for _ in 0..1000 {
            let mut rng = rand::rng();
            let choice: f32 = rng.random();

            if entries.len() == 0 || choice > 0.1 {
                let value: u128 = rng.random();

                println!("Inserting value: {}...", value);
                insert(&mut test_vec, value);
                entries.push(allocator.insert(value));

                let expected = VecAllocator::from_raw(&test_vec);
                compare_vecs(&expected, &allocator)?;
                test_iter(allocator.clone(), &test_vec)?;
            } else {
                let index = rng.random_range(0..entries.len());
                let entry = entries.remove(index);

                println!("Removing at index: {}...", entry.index);
                remove(&mut test_vec, entry.index);
                allocator.remove(entry).unwrap();

                let expected = VecAllocator::from_raw(&test_vec);
                compare_vecs(&expected, &allocator)?;
                test_iter(allocator.clone(), &test_vec)?;
            }
        }

        Ok(())
    }
}