use std::{collections::HashMap, hash::Hash};

pub struct OrderedMap<K, I>
where
    K: Hash,
{
    keys: HashMap<K, usize>,
    items: Vec<I>,
}
