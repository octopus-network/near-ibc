use crate::*;

/// A simple implementation of `indexed ordered queue`.
///
/// The implementation uses a lookup maps to store the data. In `index_map`, the keys are
/// continuous index, from `start_index` to `end_index`. The `start_index` and `end_index`
/// are used to record the valid index range of the queue.
///
/// Notes:
/// - This implementation ensures that the elements are added to the queue in order (the latest
/// added element has the largest key).
/// - The K should be a value type. If using a type with extra storage usage in K,
/// when remove them from the queue, the extra storage usage will not be released.
///
#[derive(BorshDeserialize, BorshSerialize)]
pub struct IndexedOrderedQueue<K>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
{
    /// The map of index to K.
    index_map: LookupMap<u64, K>,
    ///
    index_map_storage_key_prefix: StorageKey,
    /// The start index of valid anchor event.
    start_index: u64,
    /// The end index of valid anchor event.
    end_index: u64,
    /// The max length of queue.
    max_length: u64,
}

/// Implement change functions for `IndexedLookupQueue`.
impl<K> IndexedOrderedQueue<K>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
{
    ///
    pub fn new(
        index_map_storage_key: StorageKey,
        max_length: u64,
    ) -> Self {
        Self {
            index_map: LookupMap::new(index_map_storage_key.clone()),
            index_map_storage_key_prefix: index_map_storage_key,
            start_index: 0,
            end_index: 0,
            max_length,
        }
    }
    ///
    pub fn migrate_from(
        index_map_storage_key: StorageKey,
        start_index: u64,
        end_index: u64,
        max_length: u64,
    ) -> Self {
        Self {
            index_map: LookupMap::new(index_map_storage_key.clone()),
            index_map_storage_key_prefix: index_map_storage_key,
            start_index,
            end_index,
            max_length,
        }
    }
    /// Remove the first element from the queue and return it.
    pub fn pop_front(&mut self) -> Option<K> {
        if self.index_map.contains_key(&self.start_index) {
            let key = self.index_map.remove(&self.start_index);
            self.start_index += 1;
            key
        } else {
            None
        }
    }
    /// Add a new element to the queue.
    /// If the queue reaches max length, the oldest (first) element will be removed.
    pub fn push_back(&mut self, element: K) {
        assert!(
            self.end_index == 0 || &element > self.index_map.get(&self.end_index).unwrap(),
            "The key to be added should be larger than the latest key in the queue."
        );
        self.index_map.insert(self.end_index + 1, element.clone());
        if self.start_index == 0 && self.end_index == 0 {
            self.start_index = 1;
        }
        self.end_index += 1;
        if self.end_index - self.start_index + 1 > self.max_length {
            self.pop_front();
        }
    }
    /// Set max length of the queue.
    pub fn set_max_length(&mut self, max_length: u64) {
        self.max_length = max_length;
        while self.end_index - self.start_index + 1 > self.max_length {
            self.pop_front();
        }
    }
    /// Clear the queue.
    pub fn clear(&mut self) {
        for index in self.start_index..self.end_index + 1 {
            if let Some(key) = self.index_map.get(&index) {
                env::storage_remove(
                    migration::get_storage_key_of_lookup_map(
                        &self.index_map_storage_key_prefix,
                        &key,
                    )
                    .as_slice(),
                );
            }
        }
        self.start_index = 0;
        self.end_index = 0;
    }
}

/// Implement view functions for `IndexedLookupQueue`.
impl<K> IndexedOrderedQueue<K>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
{
    ///
    pub fn start_index(&self) -> u64 {
        self.start_index
    }
    ///
    pub fn end_index(&self) -> u64 {
        self.end_index
    }
    ///
    pub fn max_length(&self) -> u64 {
        self.max_length
    }
    ///
    pub fn latest_key(&self) -> Option<&K> {
        self.index_map.get(&self.end_index)
    }
    /// Get current length of the queue.
    pub fn len(&self) -> u64 {
        if self.end_index < self.start_index {
            return 0;
        } else if self.end_index == self.start_index {
            if self.index_map.contains_key(&self.start_index) {
                return 1;
            } else {
                return 0;
            }
        } else if self.end_index - self.start_index + 1 > self.max_length {
            return self.max_length;
        } else {
            return self.end_index - self.start_index + 1;
        }
    }
    /// Get the keys stored in the queue.
    pub fn keys(&self) -> Vec<Option<&K>> {
        let mut keys = Vec::<Option<&K>>::new();
        for index in self.start_index..self.end_index + 1 {
            keys.push(self.index_map.get(&index));
        }
        keys
    }
    /// Get key by index.
    pub fn get_key(&self, index: &u64) -> Option<&K> {
        self.index_map.get(index)
    }
    /// Get a slice of key-value pairs from the queue.
    pub fn get_slice_of(&self, start_index: &u64, quantity: Option<u64>) -> Vec<Option<&K>> {
        let mut results = Vec::<Option<&K>>::new();
        let start_index = match self.start_index > *start_index {
            true => self.start_index,
            false => *start_index,
        };
        let mut end_index = start_index
            + match quantity {
                Some(quantity) => match quantity > 50 {
                    true => 49,
                    false => quantity - 1,
                },
                None => 49,
            };
        end_index = match end_index < self.end_index {
            true => end_index,
            false => self.end_index,
        };
        for index in start_index..end_index + 1 {
            results.push(self.get_key(&index))
        }
        results
    }
    ///
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// As the queue is already ascending sorted while adding elements,
    /// so we can use binary search to find the index of the given key.
    pub fn get_index_of_key(&self, key: &K) -> Option<u64> {
        let mut start = self.start_index;
        let mut end = self.end_index;
        while start <= end {
            let mid = start + (end - start) / 2;
            let mid_key = self.index_map.get(&mid).unwrap();
            if mid_key == key {
                return Some(mid);
            } else if mid_key < key {
                start = mid + 1;
            } else {
                end = mid - 1;
            }
        }
        None
    }
    /// Get the maximum key less than the given key.
    pub fn get_previous_by_key(&self, key: &K) -> Option<&K> {
        if let Some(index) = self.get_index_of_key(key) {
            if index > self.start_index {
                self.index_map.get(&(index - 1))
            } else {
                None
            }
        } else {
            // Get the maximum key less than the given key by binary search.
            let mut start = self.start_index;
            let mut end = self.end_index;
            let mut result = None;
            while start <= end {
                let mid = start + (end - start) / 2;
                let mid_key = self.index_map.get(&mid).unwrap();
                if mid_key < key {
                    result = Some(mid_key);
                    start = mid + 1;
                } else {
                    end = mid - 1;
                }
            }
            result
        }
    }
    /// Get the minimum key greater than the given key.
    pub fn get_next_by_key(&self, key: &K) -> Option<&K> {
        if let Some(index) = self.get_index_of_key(key) {
            if index < self.end_index {
                self.index_map.get(&(index + 1))
            } else {
                None
            }
        } else {
            // Get the minimum key greater than the given key by binary search.
            let mut start = self.start_index;
            let mut end = self.end_index;
            let mut result = None;
            while start <= end {
                let mid = start + (end - start) / 2;
                let mid_key = self.index_map.get(&mid).unwrap();
                if mid_key > key {
                    result = Some(mid_key);
                    end = mid - 1;
                } else {
                    start = mid + 1;
                }
            }
            result
        }
    }
}
