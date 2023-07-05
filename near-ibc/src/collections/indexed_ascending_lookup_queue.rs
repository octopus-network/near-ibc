use crate::*;

use super::IndexedAscendingQueueViewer;

/// A indexed ascending lookup queue.
///
/// The implementation uses two lookup maps to store the data. In `index_map`, the keys are
/// continuous index, from `start_index` to `end_index`. And in `value_map`, it directly stores
/// the key-value pairs. The `start_index` and `end_index` are used to record the valid
/// index range of the queue.
///
/// The K and V should be a value type. If using a type with extra storage usage in K or V,
/// when remove them from the queue, the extra storage usage will not be released.
///
#[derive(BorshDeserialize, BorshSerialize)]
pub struct IndexedAscendingLookupQueue<K, V>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
    V: BorshDeserialize + BorshSerialize + Clone,
{
    /// The map of index to K.
    index_map: LookupMap<u64, K>,
    ///
    index_map_storage_key_prefix: StorageKey,
    /// The map of K to V.
    value_map: LookupMap<K, V>,
    ///
    value_map_storage_key_prefix: StorageKey,
    /// The start index of valid anchor event.
    start_index: u64,
    /// The end index of valid anchor event.
    end_index: u64,
    /// The max length of queue.
    max_length: u64,
}

/// Implement change functions for `IndexedLookupQueue`.
impl<K, V> IndexedAscendingLookupQueue<K, V>
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
            index_map: LookupMap::new(index_map_storage_key.clone()),
            index_map_storage_key_prefix: index_map_storage_key,
            value_map: LookupMap::new(value_map_storage_key.clone()),
            value_map_storage_key_prefix: value_map_storage_key,
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
            index_map: LookupMap::new(index_map_storage_key.clone()),
            index_map_storage_key_prefix: index_map_storage_key,
            value_map: LookupMap::new(value_map_storage_key.clone()),
            value_map_storage_key_prefix: value_map_storage_key,
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
        assert!(
            self.end_index == 0 || &element.0 > self.index_map.get(&self.end_index).unwrap(),
            "The key to be added should be larger than the latest key in the queue."
        );
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
    /// Get value by key.
    pub fn get_value_by_key_mut(&mut self, key: &K) -> Option<&mut V> {
        self.value_map.get_mut(key)
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
                    migration::get_storage_key_of_lookup_map(
                        &self.index_map_storage_key_prefix,
                        &key,
                    )
                    .as_slice(),
                );
                env::storage_remove(
                    migration::get_storage_key_of_lookup_map(
                        &self.value_map_storage_key_prefix,
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

impl<K, V> IndexedAscendingQueueViewer<K> for IndexedAscendingLookupQueue<K, V>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
    V: BorshDeserialize + BorshSerialize + Clone,
{
    ///
    fn start_index(&self) -> u64 {
        self.start_index
    }
    ///
    fn end_index(&self) -> u64 {
        self.end_index
    }
    ///
    fn max_length(&self) -> u64 {
        self.max_length
    }
    ///
    fn get_key_by_index(&self, index: &u64) -> Option<&K> {
        self.index_map.get(index)
    }
}

impl<K, V> IndexedAscendingLookupQueue<K, V>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
    V: BorshDeserialize + BorshSerialize + Clone,
{
    ///
    pub fn values(&self) -> Vec<Option<&V>> {
        let mut values = Vec::<Option<&V>>::new();
        for index in self.start_index..self.end_index + 1 {
            values.push(
                self.index_map
                    .get(&index)
                    .map(|k| self.value_map.get(k).map(|v| v).unwrap()),
            );
        }
        values
    }
    /// Get value by index.
    pub fn get_value_by_index(&self, index: &u64) -> Option<&V> {
        self.index_map
            .get(index)
            .map(|k| self.value_map.get(k).map(|v| v).unwrap())
    }
    /// Get value by key.
    pub fn get_value_by_key(&self, key: &K) -> Option<&V> {
        self.value_map.get(key)
    }
    /// Get the value of the maximum key less than the given key.
    pub fn get_previous_value_by_key(&self, key: &K) -> Option<&V> {
        if let Some(key) = self.get_previous_key_by_key(key) {
            self.get_value_by_key(&key)
        } else {
            None
        }
    }
    /// Get the value of the minimum key greater than the given key.
    pub fn get_next_value_by_key(&self, key: &K) -> Option<&V> {
        if let Some(key) = self.get_next_key_by_key(key) {
            self.get_value_by_key(&key)
        } else {
            None
        }
    }
}
