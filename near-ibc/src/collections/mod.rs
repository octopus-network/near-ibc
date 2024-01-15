use crate::prelude::*;

pub use indexed_ascending_lookup_queue::IndexedAscendingLookupQueue;
pub use indexed_ascending_simple_queue::IndexedAscendingSimpleQueue;

mod indexed_ascending_lookup_queue;
mod indexed_ascending_simple_queue;

/// View interfaces for indexed ascending queue.
///
/// The queue can be accessed by index or key.
/// The viewer assumes that the queue is sorted in ascending order.
pub trait IndexedAscendingQueueViewer<K>
where
    K: Clone + Ord,
{
    ///
    fn start_index(&self) -> u64;
    ///
    fn end_index(&self) -> u64;
    ///
    fn max_length(&self) -> u64;
    ///
    fn get_key_by_index(&self, index: &u64) -> Option<&K>;
    ///
    fn first_key(&self) -> Option<&K> {
        self.get_key_by_index(&self.start_index())
    }
    ///
    fn last_key(&self) -> Option<&K> {
        self.get_key_by_index(&self.end_index())
    }
    ///
    fn contains_key(&self, key: &K) -> bool {
        self.get_index_of_key(key).is_some()
    }
    /// Get current length of the queue.
    fn len(&self) -> u64 {
        if self.end_index() < self.start_index() {
            return 0;
        } else if self.end_index() == self.start_index() {
            if self.get_key_by_index(&self.start_index()).is_some() {
                return 1;
            } else {
                return 0;
            }
        } else if self.end_index() - self.start_index() + 1 > self.max_length() {
            return self.max_length();
        } else {
            return self.end_index() - self.start_index() + 1;
        }
    }
    /// Get the keys stored in the queue.
    fn keys(&self) -> Vec<Option<&K>> {
        let mut keys = Vec::<Option<&K>>::new();
        for index in self.start_index()..self.end_index() + 1 {
            keys.push(self.get_key_by_index(&index));
        }
        keys
    }
    ///
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Use binary search to find the index of the given key.
    ///
    /// This function assumes that the queue is sorted in ascending order.
    fn get_index_of_key(&self, key: &K) -> Option<u64> {
        let mut start = self.start_index();
        let mut end = self.end_index();
        while start <= end {
            let mid = start + (end - start) / 2;
            if let Some(mid_key) = self.get_key_by_index(&mid) {
                if mid_key == key {
                    return Some(mid);
                } else if mid_key < key {
                    start = mid + 1;
                } else {
                    end = mid - 1;
                }
            } else {
                return None;
            }
        }
        None
    }
    /// Get the maximum key less than the given key.
    ///
    /// This function assumes that the queue is sorted in ascending order.
    fn get_previous_key_by_key(&self, key: &K) -> Option<&K> {
        if let Some(index) = self.get_index_of_key(key) {
            if index > self.start_index() {
                self.get_key_by_index(&(index - 1))
            } else {
                None
            }
        } else {
            // Get the maximum key less than the given key by binary search.
            let mut start = self.start_index();
            let mut end = self.end_index();
            let mut result = None;
            while start <= end {
                let mid = start + (end - start) / 2;
                if let Some(mid_key) = self.get_key_by_index(&mid) {
                    if mid_key < key {
                        result = Some(mid_key);
                        start = mid + 1;
                    } else {
                        end = mid - 1;
                    }
                } else {
                    return None;
                }
            }
            result
        }
    }
    /// Get the minimum key greater than the given key.
    ///
    /// This function assumes that the queue is sorted in ascending order.
    fn get_next_key_by_key(&self, key: &K) -> Option<&K> {
        if let Some(index) = self.get_index_of_key(key) {
            if index < self.end_index() {
                self.get_key_by_index(&(index + 1))
            } else {
                None
            }
        } else {
            // Get the minimum key greater than the given key by binary search.
            let mut start = self.start_index();
            let mut end = self.end_index();
            let mut result = None;
            while start <= end {
                let mid = start + (end - start) / 2;
                if let Some(mid_key) = self.get_key_by_index(&mid) {
                    if mid_key > key {
                        result = Some(mid_key);
                        end = mid - 1;
                    } else {
                        start = mid + 1;
                    }
                } else {
                    return None;
                }
            }
            result
        }
    }
}
