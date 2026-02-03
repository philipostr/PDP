#![allow(dead_code)]

pub mod two_way_iterator;

pub use two_way_iterator::TwoWayIterator;

use std::collections::HashMap;

pub type Map<T> = HashMap<String, T>;
