use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use crate::parser::building_blocks::{Asop, Keyword, Op};
use crate::parser::ptag::{AstNode, OperationTree};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Marker {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug)]
pub struct MarkedComponent<T>
where
    T: Debug,
{
    pub comp: T,
    pub mark: Marker,
}

pub type MarkedString = MarkedComponent<String>;
pub type MarkedNumber = MarkedComponent<f64>;
pub type MarkedBoolean = MarkedComponent<bool>;
pub type MarkedOp = MarkedComponent<Op>;
pub type MarkedAsop = MarkedComponent<Asop>;
pub type MarkedKeyword = MarkedComponent<Keyword>;
pub type MarkedOperationTree = MarkedComponent<OperationTree>;
pub type MarkedAstNode = MarkedComponent<AstNode>;

impl<T> Clone for MarkedComponent<T>
where
    T: Debug + Clone,
{
    fn clone(&self) -> Self {
        Self {
            comp: self.comp.clone(),
            mark: self.mark,
        }
    }
}

impl<T> Copy for MarkedComponent<T> where T: Debug + Copy {}

impl<T> Display for MarkedComponent<T>
where
    T: Debug + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.comp, f)
    }
}

impl<T> PartialEq for MarkedComponent<T>
where
    T: Debug + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.comp == other.comp
    }
}

impl<T> Eq for MarkedComponent<T> where T: Debug + Eq {}

impl<T> Deref for MarkedComponent<T>
where
    T: Debug,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.comp
    }
}

impl<T> DerefMut for MarkedComponent<T>
where
    T: Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.comp
    }
}

impl<T> Hash for MarkedComponent<T>
where
    T: Debug + Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.comp.hash(state);
    }
}

impl<T> MarkedComponent<T>
where
    T: Debug,
{
    pub fn new(comp: T, mark: Marker) -> Self {
        Self { comp, mark }
    }
}
