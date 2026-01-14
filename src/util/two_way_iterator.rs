use std::ops::{Index, RangeFull};

#[derive(Debug)]
pub enum Direction {
    Forwards,
    Backwards,
}

#[derive(Debug)]
pub struct TwoWayIterator<'a, T> {
    source: &'a [T],
    cursor: usize,
    direction: Direction,
}

impl<'a, T> Iterator for TwoWayIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor == self.source.len() {
            None
        } else {
            let out = Some(&self.source[self.cursor]);
            self.cursor += 1;
            out
        }
    }
}

impl<'a, T> TwoWayIterator<'a, T> {
    pub fn from_source<I>(source: &'a I) -> Self
    where
        I: Index<RangeFull, Output = [T]>,
    {
        Self {
            source: &source[..],
            cursor: 0,
            direction: Direction::Forwards,
        }
    }

    pub fn dir_next(&mut self) -> Option<&T> {
        match self.direction {
            Direction::Forwards => self.next(),
            Direction::Backwards => self.rev(),
        }
    }

    pub fn dir_rev(&mut self) -> Option<&T> {
        match self.direction {
            Direction::Forwards => self.rev(),
            Direction::Backwards => self.next(),
        }
    }

    pub fn dir_next_nth(&mut self, n: usize) -> Option<&T> {
        match self.direction {
            Direction::Forwards => self.nth(n),
            Direction::Backwards => self.rev_nth(n),
        }
    }

    pub fn dir_rev_nth(&mut self, n: usize) -> Option<&T> {
        match self.direction {
            Direction::Forwards => self.rev_nth(n),
            Direction::Backwards => self.nth(n),
        }
    }

    pub fn flip(&mut self) {
        self.direction = match self.direction {
            Direction::Forwards => Direction::Backwards,
            Direction::Backwards => Direction::Forwards,
        };
    }

    /// Peek at the next value without moving the iterator.
    pub fn peek(&self) -> Option<&T> {
        if self.cursor == self.source.len() - 1 {
            None
        } else {
            Some(&self.source[self.cursor])
        }
    }

    /// Peek at the previous value without moving the iterator.
    pub fn prev(&self) -> Option<&T> {
        if self.cursor == 0 {
            None
        } else {
            Some(&self.source[self.cursor - 1])
        }
    }

    /// Reverse the iterator by a single step, and return the crossed value.
    pub fn rev(&mut self) -> Option<&T> {
        self.rev_nth(1)
    }

    /// Reverse the iterator by `n` steps, and return the last crossed value.
    pub fn rev_nth(&mut self, n: usize) -> Option<&T> {
        if self.cursor - n + 1 == 0 {
            self.cursor = 0;
            None
        } else {
            self.cursor -= n;
            Some(&self.source[self.cursor])
        }
    }
}
