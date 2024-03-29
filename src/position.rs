use std::{fmt::{Debug, Display}, ops::Range};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Position {
    pub ln: Range<usize>,
    pub col: Range<usize>,
}
pub struct Located<T> {
    pub value: T,
    pub pos: Position
}

impl Position {
    pub fn new(ln: Range<usize>, col: Range<usize>) -> Self {
        Self { ln, col }
    }
    pub fn extend(&mut self, other: &Self) {
        self.ln.end = other.ln.end;
    }
}
impl<T> Located<T> {
    pub fn new(value: T, pos: Position) -> Self {
        Self { value, pos }
    }
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> Located<U> {
        Located { value: f(self.value), pos: self.pos }
    }
    pub fn unwrap(self) -> T {
        self.value
    }
}
impl<T: Debug> Debug for Located<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T: Display> Display for Located<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T: Clone> Clone for Located<T> {
    fn clone(&self) -> Self {
        Self { value: self.value.clone(), pos: self.pos.clone() }
    }
}
impl<T: PartialEq> PartialEq for Located<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}