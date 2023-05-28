use crate::*;

/// A simple implementation of `indexed lookup queue`.
///
/// The implementation uses two lookup maps to store the data. In `index_map`, the keys are
/// continuous index, from `start_index` to `end_index`. And in `value_map`, it directly stores
/// the key-value pairs. The `start_index` and `end_index` are used to record the valid
/// index range of the queue.
///
/// Notes:
/// - This implementation assumes that the elements are added to the queue in order (the latest
/// added element has the largest key). Otherwise, the `get_previous_by_key` and
/// `get_next_by_key` functions will not work correctly.
/// - The K and V should be a value type. If using a type with extra storage usage in K or V,
/// when remove them from the queue, the extra storage usage will not be released.
///
#[derive(BorshDeserialize, BorshSerialize)]
pub struct IndexedLookupQueue<K, V>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
    V: BorshDeserialize + BorshSerialize + Clone,
{
    /// The map of index to K.
    index_map: LookupMap<u64, K>,
    /// The map of K to V.
    value_map: LookupMap<K, V>,
    /// The start index of valid anchor event.
    start_index: u64,
    /// The end index of valid anchor event.
    end_index: u64,
    /// The max length of queue.
    max_length: u64,
}

/// Implement change functions for `IndexedLookupQueue`.
impl<K, V> IndexedLookupQueue<K, V>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
    V: BorshDeserialize + BorshSerialize + Clone,
{
    ///
    pub fn new(
        index_map_storage_key: StorageKey,
        value_map_storage_key: StorageKey,
        max_length: u64,
    ) -> Self {
        Self {
            index_map: LookupMap::new(index_map_storage_key),
            value_map: LookupMap::new(value_map_storage_key),
            start_index: 0,
            end_index: 0,
            max_length,
        }
    }
    ///
    pub fn migrate_from(
        index_map_storage_key: StorageKey,
        value_map_storage_key: StorageKey,
        start_index: u64,
        end_index: u64,
        max_length: u64,
    ) -> Self {
        Self {
            index_map: LookupMap::new(index_map_storage_key),
            value_map: LookupMap::new(value_map_storage_key),
            start_index,
            end_index,
            max_length,
        }
    }
    /// Remove the first element from the queue and return it.
    pub fn pop_front(&mut self) -> Option<(Option<K>, Option<V>)> {
        if self.index_map.contains_key(&self.start_index) {
            let key = self.index_map.remove(&self.start_index);
            let value = self.value_map.remove(&key.clone().unwrap());
            self.start_index += 1;
            Some((key, value))
        } else {
            None
        }
    }
    /// Add a new element to the queue.
    /// If the queue reaches max length, the oldest (first) element will be removed.
    pub fn push_back(&mut self, element: (K, V)) {
        self.index_map.insert(self.end_index + 1, element.0.clone());
        self.value_map.insert(element.0, element.1);
        if self.start_index == 0 && self.end_index == 0 {
            self.start_index = 1;
        }
        self.end_index += 1;
        if self.end_index - self.start_index + 1 > self.max_length {
            self.pop_front();
        }
    }
    /// Remove a value from the queue by key but keep the index.
    pub fn remove_by_key(&mut self, key: &K) -> Option<V> {
        self.value_map.remove(key)
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
                    migration::get_storage_key_in_lookup_array(
                        &StorageKey::IbcEventsHistoryKey,
                        &key,
                    )
                    .as_slice(),
                );
                env::storage_remove(
                    migration::get_storage_key_in_lookup_array(
                        &StorageKey::IbcEventsHistoryIndex,
                        &index,
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
impl<K, V> IndexedLookupQueue<K, V>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
    V: BorshDeserialize + BorshSerialize + Clone,
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
    pub fn keys(&self) -> Vec<Option<K>> {
        let mut keys = Vec::<Option<K>>::new();
        for index in self.start_index..self.end_index + 1 {
            keys.push(self.index_map.get(&index).map(|k| k.clone()));
        }
        keys
    }
    /// Get the values stored in the queue.
    pub fn values(&self) -> Vec<Option<V>> {
        let mut values = Vec::<Option<V>>::new();
        for index in self.start_index..self.end_index + 1 {
            values.push(
                self.index_map
                    .get(&index)
                    .map(|k| self.value_map.get(k).map(|v| v.clone()).unwrap()),
            );
        }
        values
    }
    /// Get key by index.
    pub fn get_key(&self, index: &u64) -> Option<K> {
        self.index_map.get(index).map(|k| k.clone())
    }
    /// Get value by index.
    pub fn get_value_by_index(&self, index: &u64) -> Option<V> {
        self.index_map
            .get(index)
            .map(|k| self.value_map.get(k).map(|v| v.clone()).unwrap())
    }
    /// Get value by key.
    pub fn get_value_by_key(&self, key: &K) -> Option<V> {
        self.value_map.get(key).map(|v| v.clone())
    }
    /// Get a slice of key-value pairs from the queue.
    pub fn get_slice_of(
        &self,
        start_index: &u64,
        quantity: Option<u64>,
    ) -> Vec<(Option<K>, Option<V>)> {
        let mut results = Vec::<(Option<K>, Option<V>)>::new();
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
            results.push((self.get_key(&index), self.get_value_by_index(&index)))
        }
        results
    }
    ///
    pub fn contains_key(&self, key: &K) -> bool {
        self.value_map.contains_key(key)
    }
    ///
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// We assume that the queue is already ascending sorted,
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
    /// Get the value that the corresponding key's index is 1 smaller than the given key's index.
    pub fn get_previous_by_key(&self, key: &K) -> Option<V> {
        if let Some(index) = self.get_index_of_key(key) {
            if index > self.start_index {
                self.get_value_by_index(&(index - 1)).map(|v| v.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    /// Get the value that the corresponding key's index is 1 bigger than the given key's index.
    pub fn get_next_by_key(&self, key: &K) -> Option<V> {
        if let Some(index) = self.get_index_of_key(key) {
            if index < self.end_index {
                self.get_value_by_index(&(index + 1)).map(|v| v.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}
