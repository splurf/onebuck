use std::{
    fmt::Debug,
    ops::Deref,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Manages the capacity of a dynamic data structure.
///
/// Tracks the original and current capacity and provides methods to adjust the capacity.
#[derive(Clone, Debug)]
struct Capacity {
    original: usize,
    current: usize,
}

impl Capacity {
    /// Creates a new `Capacity` with the given initial size.
    const fn new(original: usize) -> Self {
        Self {
            original,
            current: original,
        }
    }

    /// Reduces the current capacity by the original size.
    pub fn shrink(&mut self) {
        self.current -= self.original;
    }

    /// Increases the current capacity by the original size.
    pub fn grow(&mut self) {
        self.current += self.original;
    }
}

/// Represents an index in a data structure.
///
/// `ValueIndex` is used to uniquely identify a position in the data structure.
/// It provides access to elements stored in a `Bucket`.
#[derive(Debug)]
pub struct ValueIndex(Rc<AtomicUsize>);

pub struct Value<T> {
    data: T,
    index: Rc<AtomicUsize>,
}

#[cfg(feature = "clone")]
impl Clone for ValueIndex {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(feature = "clone")]
impl<T> Value<T> {
    pub fn index(&self) -> ValueIndex {
        ValueIndex(self.index.clone())
    }
}

impl<T> Deref for Value<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Debug> Debug for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.data))
    }
}

/// A dynamic array-like data structure that supports efficient insertion, removal, and capacity management.
#[derive(Debug)]
pub struct Bucket<T> {
    data: Vec<Value<T>>,
    capacity: Capacity,
}

impl<T> Bucket<T> {
    /// Creates a new `Bucket` with the specified initial capacity.
    ///
    /// # Arguments
    /// * `capacity` - The initial number of slots available in the `Bucket`.
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity: Capacity::new(capacity),
        }
    }

    /// Returns the number of elements currently stored in the `Bucket`.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns the current capacity of the `Bucket`.
    pub const fn capacity(&self) -> usize {
        self.capacity.current
    }

    /// Checks if the `Bucket` is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns an iterator over the values in the `Bucket`.
    #[cfg(feature = "clone")]
    pub fn iter(&self) -> impl Iterator<Item = &Value<T>> {
        self.data.iter()
    }

    /// Returns an iterator over the elements in the `Bucket`.
    #[cfg(not(feature = "clone"))]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().map(|v| &v.data)
    }

    /// Retrieves a reference to the value at the given index.
    ///
    /// # Arguments
    /// * `index` - The `ValueIndex` of the value to retrieve.
    #[cfg(feature = "get")]
    pub fn get(&self, index: &ValueIndex) -> &T {
        &self.data[index.0.load(Ordering::Relaxed)].data
    }

    /// Inserts a new value into the `Bucket`.
    ///
    /// If the `Bucket` is full, it will automatically grow to accommodate the new value.
    ///
    /// # Arguments
    /// * `data` - The value to insert.
    pub fn insert(&mut self, data: T) -> ValueIndex {
        if self.len() == self.capacity.current {
            self.grow()
        }
        let index = self.data.len();
        let index_shared = Rc::new(AtomicUsize::new(index));

        self.data.push(Value {
            data,
            index: index_shared.clone(),
        });

        ValueIndex(index_shared)
    }

    /// Removes the value at the specified index.
    ///
    /// The slot is freed for future use, and the internal array may be compacted.
    ///
    /// # Arguments
    /// * `index` - The `ValueIndex` of the value to remove.
    #[cfg(not(feature = "clone"))]
    pub fn remove(&mut self, index: ValueIndex) -> T {
        let index = index.0.load(Ordering::Relaxed);
        self._remove(index)
    }

    /// Removes the value at the specified index, if it exists.
    ///
    /// The slot is freed for future use, and the internal array may be compacted.
    ///
    /// # Arguments
    /// * `index` - The `ValueIndex` of the value to remove.
    #[cfg(feature = "clone")]
    pub fn remove(&mut self, index: ValueIndex) -> Option<T> {
        let index = index.0.load(Ordering::Relaxed);
        self.data.get(index).is_some().then(|| self._remove(index))
    }

    fn _remove(&mut self, i: usize) -> T {
        let j = self.len() - 1;

        if self.len() > 1 && i < j {
            // Swap with the last element
            self.data.swap(i, j);

            // Update the index of the swapped element
            self.data[i].index.store(i, Ordering::Relaxed)
        }

        // Remove and return the element at the index
        let value = {
            #[cfg(test)]
            {
                self.data.pop().unwrap()
            }

            #[cfg(not(test))]
            unsafe {
                self.data.pop().unwrap_unchecked()
            }
        };

        // Shrink the capacity if needed
        if j > 0 && j == self.capacity.current - self.capacity.original {
            self.shrink()
        }
        value.data
    }

    /// Increases the capacity of the `Bucket`.
    ///
    /// This method is called internally when the `Bucket` is full.
    fn grow(&mut self) {
        self.capacity.grow();
        self.data.reserve(self.capacity.original)
    }

    /// Reduces the capacity of the `Bucket`.
    ///
    /// This method is called internally when elements are removed.
    fn shrink(&mut self) {
        self.capacity.shrink();
        self.data.shrink_to(self.capacity.current)
    }
}

impl<T> Default for Bucket<T> {
    /// Creates a `Bucket` with a default capacity of `32`.
    fn default() -> Self {
        Self::new(32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let bucket = Bucket::<u8>::new(10);
        assert_eq!(bucket.len(), 0);
        assert!(bucket.is_empty());
    }

    #[test]
    fn test_insert() {
        let mut bucket = Bucket::new(2);
        let idx1 = bucket.insert(42);
        let idx2 = bucket.insert(43);
        assert_eq!(*bucket.get(&idx1), 42);
        assert_eq!(*bucket.get(&idx2), 43);
    }

    #[test]
    fn test_remove() {
        let mut bucket = Bucket::new(2);
        let idx = bucket.insert(42);
        let value = bucket.remove(idx);

        #[cfg(not(feature = "clone"))]
        assert_eq!(value, 42);

        #[cfg(feature = "clone")]
        assert_eq!(value, Some(42));

        assert!(bucket.is_empty());
    }

    #[test]
    fn test_capacity_growth() {
        let mut bucket = Bucket::new(2);
        bucket.insert(1);
        bucket.insert(2);
        bucket.insert(3); // Triggers growth
        assert_eq!(bucket.len(), 3);
    }

    #[test]
    fn test_capacity_shrink() {
        let mut bucket = Bucket::new(10);
        for i in 0..10 {
            bucket.insert(i);
        }
        bucket.capacity.shrink();
        assert_eq!(bucket.capacity(), 0);
    }

    #[test]
    fn test_edge_cases() {
        let mut bucket = Bucket::new(1);
        let idx = bucket.insert(10);
        assert_eq!(*bucket.get(&idx), 10);
        bucket.remove(idx);
        assert!(bucket.is_empty());
    }

    #[cfg(not(feature = "clone"))]
    #[test]
    #[should_panic]
    fn test_remove_empty() {
        let mut bucket = Bucket::new(1);
        let idx = bucket.insert(1);
        let idx_clone = ValueIndex(idx.0.clone());
        bucket.remove(idx);
        bucket.remove(idx_clone); // Should panic as index is invalid
    }

    #[cfg(feature = "clone")]
    #[test]
    fn test_remove_empty() {
        let mut bucket = Bucket::new(1);
        let idx = bucket.insert(1);
        let idx_clone = ValueIndex(idx.0.clone());
        bucket.remove(idx);
        assert_eq!(bucket.remove(idx_clone), None)
    }

    #[test]
    fn test_capacity_management() {
        let mut bucket = Bucket::new(2);
        let a = bucket.insert(1);
        let b = bucket.insert(2);
        let c = bucket.insert(3); // Should trigger growth

        assert_eq!(bucket.capacity(), 4);
        assert_eq!(bucket.len(), 3);

        bucket.remove(a);
        bucket.remove(b);
        bucket.remove(c); // Should trigger shrinking

        assert_eq!(bucket.capacity(), 2);
    }

    #[test]
    fn test_iter_after_removal() {
        let mut bucket = Bucket::new(5);
        let idx1 = bucket.insert(1);
        _ = bucket.insert(2);
        bucket.remove(idx1);

        #[cfg(not(feature = "clone"))]
        let values: Vec<_> = bucket.iter().cloned().collect();

        #[cfg(feature = "clone")]
        let values: Vec<_> = bucket.iter().map(|v| v.data).collect();

        assert_eq!(values, vec![2]);
    }

    #[test]
    fn test_repeated_inserts_removals() {
        let mut bucket = Bucket::new(3);
        for i in 0..5 {
            let idx = bucket.insert(i);
            assert_eq!(*bucket.get(&idx), i);
            bucket.remove(idx);
        }
        assert!(bucket.is_empty());
    }
}
