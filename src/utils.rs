use std::collections::VecDeque;

/// An iterator adapter that allows peeking multiple items ahead
pub struct PeekableN<I: Iterator> {
    iter: I,
    buffer: VecDeque<I::Item>,
}

impl<I: Iterator> PeekableN<I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            buffer: VecDeque::new(),
        }
    }

    /// Check if upcoming items match a given iterator (zero-alloc)
    pub fn starts_with<'a, T>(&mut self, prefix: T) -> bool
    where
        I::Item: 'a + PartialEq + Clone,
        T: IntoIterator<Item = &'a I::Item> + Clone,
    {
        let needed = prefix.clone().into_iter().count();
        while self.buffer.len() < needed {
            if let Some(item) = self.iter.next() {
                self.buffer.push_back(item);
            } else {
                break;
            }
        }
        self.buffer.iter().zip(prefix).all(|(a, b)| a == b)
    }
}

impl<I: Iterator<Item = char>> PeekableN<I> {
    pub fn starts_with_str(&mut self, prefix: &str) -> bool {
        let needed = prefix.chars().count();
        while self.buffer.len() < needed {
            if let Some(item) = self.iter.next() {
                self.buffer.push_back(item);
            } else {
                break;
            }
        }
        self.buffer.iter().zip(prefix.chars()).all(|(a, b)| *a == b)
    }
}

impl<I: Iterator> Iterator for PeekableN<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.buffer.pop_front() {
            Some(item)
        } else {
            self.iter.next()
        }
    }
}
