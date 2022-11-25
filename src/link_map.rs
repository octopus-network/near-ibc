use crate::*;
use near_sdk::IntoStorageKey;

// todo split value and pre,next
// if split, when we find insert index, we needn't read whole value, but we'll cost more storage
#[derive(BorshSerialize, BorshDeserialize)]
struct LinkValue<K, V> {
    pub value: V,
    pub pre: Option<K>,
    pub next: Option<K>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct KeySortLinkMap<K, V> {
    lookup_map: LookupMap<K, LinkValue<K, V>>,
    head: Option<K>,
    tail: Option<K>,
}

impl<K, V> KeySortLinkMap<K, V>
where
    K: Ord + BorshSerialize + BorshDeserialize + Clone,
    V: BorshSerialize + BorshDeserialize + Clone,
{
    pub fn new<S>(key_prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        Self {
            lookup_map: LookupMap::new(key_prefix),
            head: None,
            tail: None,
        }
    }

    pub fn get(&self, k: &K) -> Option<V> {
        self.lookup_map.get(k).map(|v| v.value)
    }

    pub fn get_pre(&self, k: &K) -> Option<V> {
        self.lookup_map
            .get(k)
            .and_then(|v| v.pre)
            .and_then(|pre_k| self.get(&pre_k))
    }

    pub fn get_next(&self, k: &K) -> Option<V> {
        self.lookup_map
            .get(k)
            .and_then(|v| v.next)
            .and_then(|next_k| self.get(&next_k))
    }

    // todo whether value will be same
    pub fn insert_from_tail(&mut self, k: &K, v: &V) {
        // self
        if let Some(find_k) = self.find_first_lt_key_from_tail(k) {
            self.insert_after(&find_k, self.lookup_map.get(&find_k).unwrap(), k, v)
        } else {
            self.insert_as_head(k, v)
        }
    }

    fn find_first_lt_key_from_tail(&self, k: &K) -> Option<K> {
        let mut current_k = self.tail.clone();

        while current_k.is_some() && k.lt(&current_k.as_ref().unwrap()) {
            current_k = self
                .lookup_map
                .get(&current_k.unwrap())
                .unwrap()
                .pre
                .clone();
        }

        current_k
    }

    fn insert_as_head(&mut self, k: &K, v: &V) {
        if let Some(old_head_k) = &self.head {
            //1. update old head
            let mut old_head_link_v = self.lookup_map.get(k).unwrap();
            old_head_link_v.pre = Some(old_head_k.clone());
            self.lookup_map.insert(old_head_k, &old_head_link_v);

            //2. insert new head
            self.internal_insert(
                k,
                &LinkValue {
                    value: v.clone(),
                    pre: None,
                    next: Some(old_head_k.clone()),
                },
            )
        } else {
            self.internal_insert(
                k,
                &LinkValue {
                    value: v.clone(),
                    pre: None,
                    next: None,
                },
            )
        }
    }

    fn insert_before(&mut self, k: &K, mut link_v: LinkValue<K, V>, new_k: &K, new_v: &V) {
        // 1. insert new value
        let new_link_v = LinkValue {
            value: new_v.clone(),
            pre: link_v.pre.clone(),
            next: Some(k.clone()),
        };
        self.internal_insert(k, &new_link_v);

        // 2. update and insert aim value
        link_v.pre = Some(new_k.clone());
        self.lookup_map.insert(k, &link_v);

        // 3. update and insert pre value
        if let Some(pre_key) = link_v.pre {
            let mut pre_link_v = self.lookup_map.get(&pre_key).unwrap();
            pre_link_v.next = Some(new_k.clone());
            self.lookup_map.insert(&pre_key, &pre_link_v);
        } else {
            self.head = Some(new_k.clone())
        }
    }

    fn insert_after(&mut self, k: &K, mut link_v: LinkValue<K, V>, new_k: &K, new_v: &V) {
        // 1. insert new value
        let new_link_v = LinkValue {
            value: new_v.clone(),
            pre: Some(k.clone()), //link_value.pre.clone(),
            next: link_v.next.clone(),
        };
        self.internal_insert(k, &new_link_v);

        // 2. update and insert aim value
        link_v.next = Some(new_k.clone());
        self.internal_insert(k, &link_v);

        // 3. update and insert pre value
        if let Some(next_key) = link_v.next {
            let mut next_link_v = self.lookup_map.get(&next_key).unwrap();
            next_link_v.next = Some(new_k.clone());
            self.lookup_map.insert(&next_key, &next_link_v);
        } else {
            self.tail = Some(new_k.clone())
        }
    }

    fn internal_insert(&mut self, k: &K, v: &LinkValue<K, V>) {
        self.lookup_map.insert(k, v);
        if v.pre.is_none() {
            self.head = Some(k.clone())
        }

        if v.next.is_none() {
            self.tail = Some(k.clone())
        }
    }
}
