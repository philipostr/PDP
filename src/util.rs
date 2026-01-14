#![allow(dead_code)]

pub mod ordered_map;
pub mod two_way_iterator;

pub use ordered_map::OrderedMap;
pub use two_way_iterator::TwoWayIterator;

use std::collections::HashMap;

pub type OrderedSet<V> = OrderedMap<V, ()>;
pub type Map<T> = HashMap<String, T>;
