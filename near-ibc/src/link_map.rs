use crate::*;
use near_sdk::IntoStorageKey;

// todo split value and pre,next
// if split, when we find insert index, we needn't read whole value, but we'll cost more storage
#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct Link<K> {
    pub pre_k: Option<K>,
    pub next_k: Option<K>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct KeySortLinkMap<K, V> {
    value_map: LookupMap<K, V>,
    link_map: LookupMap<K, Link<K>>,
    head: Option<K>,
    tail: Option<K>,
}

impl<K, V> KeySortLinkMap<K, V>
where
    K: Ord + BorshSerialize + BorshDeserialize + Clone,
    V: BorshSerialize + BorshDeserialize + Clone,
{
    pub fn new<S>(key_prefix: S, link_prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        Self {
            value_map: LookupMap::new(key_prefix),
            link_map: LookupMap::new(link_prefix),
            head: None,
            tail: None,
        }
    }

    pub fn get(&self, k: &K) -> Option<V> {
        self.value_map.get(k)
    }

    pub fn get_pre(&self, k: &K) -> Option<V> {
        self.link_map
            .get(k)
            .and_then(|link| link.pre_k)
            .and_then(|pre_k| self.get(&pre_k))
    }

    pub fn get_next(&self, k: &K) -> Option<V> {
        self.link_map
            .get(k)
            .and_then(|link| link.next_k)
            .and_then(|next_k| self.get(&next_k))
    }

    // todo whether value will be same
    pub fn insert_from_tail(&mut self, k: &K, v: &V) {
        // insert key-value
        self.value_map.insert(k, v);
        // insert key-link
        if let Some(find_k) = self.find_first_lt_key_from_tail(k) {
            self.insert_k_after(&find_k, self.link_map.get(&find_k).unwrap(), k)
        } else {
            self.insert_k_as_head(k)
        }
    }

    fn find_first_lt_key_from_tail(&self, k: &K) -> Option<K> {
        let mut current_k = self.tail.clone();

        while current_k.is_some() && k.lt(&current_k.as_ref().unwrap()) {
            current_k = self
                .link_map
                .get(&current_k.unwrap())
                .unwrap()
                .pre_k
                .clone();
        }

        current_k
    }

    fn insert_k_as_head(&mut self, k: &K) {
        if let Some(old_head_k) = &self.head {
            //1. update old head
            let mut old_head_link = self.link_map.get(old_head_k).unwrap();
            old_head_link.pre_k = Some(old_head_k.clone());
            self.link_map.insert(old_head_k, &old_head_link);

            //2. insert new head
            self.link_map.insert(
                k,
                &Link {
                    pre_k: None,
                    next_k: Some(old_head_k.clone()),
                },
            );
        } else {
            self.link_map.insert(
                k,
                &Link {
                    pre_k: None,
                    next_k: None,
                },
            );
        }
        self.head = Some(k.clone())
    }

    fn insert_before(&mut self, k: &K, link_v: Link<K>, new_k: &K) {
        // 1. insert new value
        let new_link = Link {
            pre_k: link_v.pre_k.clone(),
            next_k: Some(k.clone()),
        };
        self.insert_link_and_update_pre_next(new_k, &new_link);

        // self.internal_insert(k, &new_link_v);
        //
        // // 2. update and insert aim value
        // link_v.pre = Some(new_k.clone());
        // self.value_map.insert(k, &link_v);
        //
        // // 3. update and insert pre value
        // if let Some(pre_key) = link_v.pre {
        //     let mut pre_link_v = self.value_map.get(&pre_key).unwrap();
        //     pre_link_v.next = Some(new_k.clone());
        //     self.value_map.insert(&pre_key, &pre_link_v);
        // } else {
        //     self.head = Some(new_k.clone())
        // }
    }

    fn insert_k_after(&mut self, k: &K, mut link: Link<K>, new_k: &K) {
        // 1. insert new value
        let new_link = Link {
            pre_k: Some(k.clone()), //link_value.pre.clone(),
            next_k: link.next_k.clone(),
        };
        self.insert_link_and_update_pre_next(new_k, &new_link);
        // self.internal_insert(new_k, &new_link_v);

        // 2. update and insert aim value
        // link_v.next = Some(new_k.clone());
        // self.internal_insert(k, &link_v);

        // 3. update and insert next value
        // if let Some(next_key) = new_link_v.next {
        //     let mut next_link_v = self.value_map.get(&next_key).unwrap();
        //     next_link_v.next = Some(new_k.clone());
        //     self.value_map.insert(&next_key, &next_link_v);
        // } else {
        //     self.tail = Some(new_k.clone())
        // }
    }

    fn insert_link_and_update_pre_next(&mut self, k: &K, link: &Link<K>) {
        self.link_map.insert(k, link);
        if link.next_k.is_none() {
            self.head = Some(k.clone())
        } else {
            let next_k = link.next_k.clone().unwrap();
            let mut next_link = self.link_map.get(&next_k).unwrap();
            if next_link.pre_k.is_none() || next_link.pre_k.unwrap().ne(k) {
                next_link.pre_k = Some(k.clone());
                self.link_map.insert(&next_k, &next_link);
            }
        }

        if link.pre_k.is_none() {
            self.tail = Some(k.clone());
        } else {
            let pre_k = link.pre_k.clone().unwrap();
            let mut pre_link = self.link_map.get(&pre_k).unwrap();
            if pre_link.next_k.is_none() || pre_link.next_k.unwrap().ne(k) {
                pre_link.next_k = Some(k.clone());
                self.link_map.insert(&pre_k, &pre_link);
            }
        }
    }

    // fn internal_insert(&mut self, k: &K, link: &Link<K>) {
    //     self.value_map.insert(k, v);
    //     if v.pre.is_none() {
    //         self.head = Some(k.clone())
    //     }
    //
    //     if v.next.is_none() {
    //         self.tail = Some(k.clone())
    //     }
    // }
}

#[test]
fn test() {
    let mut map: KeySortLinkMap<u32, u32> = KeySortLinkMap::new(
        StorageKey::ConsensusStatesKey {
            client_id: Default::default(),
        },
        StorageKey::ConsensusStatesLink {
            client_id: Default::default(),
        },
    );
    map.insert_from_tail(&1, &2);
    map.insert_from_tail(&3, &2);
    map.insert_from_tail(&2, &2);
    let mut head = map.head;
    while head.is_some() {
        let k = head.unwrap();
        let link = map.link_map.get(&k).unwrap();
        let value = map.value_map.get(&k).unwrap();
        dbg!(&link, &value);
        head = link.next_k;
    }
}
