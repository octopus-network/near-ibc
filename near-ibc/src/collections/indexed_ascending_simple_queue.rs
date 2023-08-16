use crate::*;

use super::IndexedAscendingQueueViewer;

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
pub struct IndexedAscendingSimpleQueue<K>
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
impl<K> IndexedAscendingSimpleQueue<K>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
{
    ///
    pub fn new(index_map_storage_key: StorageKey, max_length: u64) -> Self {
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
    ///
    pub fn push_back(&mut self, key: K) {
        assert!(
            self.end_index == 0 || &key > self.get_key_by_index(&self.end_index).unwrap(),
            "The key to be added should be larger than the latest key in the queue."
        );
        self.index_map.insert(self.end_index + 1, key.clone());
        if self.start_index == 0 && self.end_index == 0 {
            self.start_index = 1;
        }
        self.end_index += 1;
        if self.end_index - self.start_index + 1 > self.max_length {
            self.pop_front();
        }
    }
    ///
    pub fn pop_front(&mut self) -> Option<K> {
        if self.index_map.contains_key(&self.start_index) {
            let key = self.index_map.remove(&self.start_index);
            self.start_index += 1;
            key
        } else {
            None
        }
    }
    ///
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
    /// Flush the lookup map to storage.
    pub fn flush(&mut self) {
        self.index_map.flush();
    }
}

impl<K> Drop for IndexedAscendingSimpleQueue<K>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
{
    fn drop(&mut self) {
        self.flush();
    }
}

impl<K> IndexedAscendingQueueViewer<K> for IndexedAscendingSimpleQueue<K>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
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
    //
    fn get_key_by_index(&self, index: &u64) -> Option<&K> {
        self.index_map.get(index)
    }
}
