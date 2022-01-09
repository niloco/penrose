//! An internal data structure and associated helpers for simplifying actions around
//! manipulating focusable ordered collections.
use crate::v3::{Error, Result};
use std::{
    collections::VecDeque,
    iter::{FromIterator, IntoIterator},
    ops::{Index, IndexMut},
};

/// A direction to rotate elements of a collection
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    /// increase the index, wrapping if needed
    Forward,
    /// decrease the index, wrapping if needed
    Backward,
}

/// Where a given element should be inserted into a Ring
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum InsertPoint {
    /// At the specified index (last if out of bounds)
    Index(usize),
    /// In place of the current focused element (pushing focused and later down in the stack)
    Focused,
    /// After the current focused element (pushing later elements down in the stack)
    AfterFocused,
    /// As the first element in the stack
    First,
    /// As the last element in the stack
    Last,
}

use Direction::*;
use InsertPoint::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct Ring<T> {
    inner: VecDeque<T>,
    pub(crate) focused: usize,
}

impl<T> From<Vec<T>> for Ring<T> {
    fn from(v: Vec<T>) -> Self {
        Self {
            inner: v.into(),
            focused: 0,
        }
    }
}

impl<T> Index<usize> for Ring<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T> IndexMut<usize> for Ring<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<T> FromIterator<T> for Ring<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            inner: VecDeque::from_iter(iter),
            focused: 0,
        }
    }
}

impl<T> IntoIterator for Ring<T> {
    type Item = T;
    type IntoIter = std::collections::vec_deque::IntoIter<T>;

    /// Consumes the `VecDeque` into a front-to-back iterator yielding elements by
    /// value.
    fn into_iter(self) -> std::collections::vec_deque::IntoIter<T> {
        self.inner.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Ring<T> {
    type Item = &'a T;
    type IntoIter = std::collections::vec_deque::Iter<'a, T>;

    fn into_iter(self) -> std::collections::vec_deque::Iter<'a, T> {
        self.inner.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Ring<T> {
    type Item = &'a mut T;
    type IntoIter = std::collections::vec_deque::IterMut<'a, T>;

    fn into_iter(self) -> std::collections::vec_deque::IterMut<'a, T> {
        self.inner.iter_mut()
    }
}

impl<T> Ring<T> {
    pub fn new() -> Self {
        Self {
            inner: VecDeque::new(),
            focused: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
            focused: 0,
        }
    }

    #[inline]
    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.inner.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> std::collections::vec_deque::IterMut<'_, T> {
        self.inner.iter_mut()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn elements(&self) -> &[T] {
        self.inner.as_slices().0
    }

    #[inline]
    pub fn focused_index(&self) -> Option<usize> {
        if self.inner.is_empty() {
            None
        } else {
            Some(self.focused)
        }
    }

    #[inline]
    pub fn focused_element(&self) -> Option<&T> {
        self.inner.get(self.focused)
    }

    #[inline]
    pub fn focused_element_unchecked(&self) -> &T {
        &self.inner[self.focused]
    }

    #[inline]
    pub fn focused_element_mut(&mut self) -> Option<&mut T> {
        self.inner.get_mut(self.focused)
    }

    #[inline]
    pub fn focused_element_mut_unchecked(&mut self) -> &mut T {
        &mut self.inner[self.focused]
    }

    #[inline]
    pub fn insert(&mut self, t: T, ip: InsertPoint) {
        match ip {
            First => self.inner.push_front(t),
            Last => self.inner.push_back(t),
            Index(i) => self.inner.insert(i, t),
            Focused => self.inner.insert(self.focused, t),
            AfterFocused => {
                let ix = self.focused + 1;
                if ix > self.inner.len() {
                    self.inner.push_back(t)
                } else {
                    self.inner.insert(ix, t)
                }
            }
        }

        self.inner.make_contiguous();
    }

    #[inline]
    pub fn rotate(&mut self, direction: Direction) {
        if self.inner.is_empty() {
            return;
        }

        match direction {
            Forward => self.inner.rotate_right(1),
            Backward => self.inner.rotate_left(1),
        }

        self.inner.make_contiguous();
    }

    pub fn would_wrap(&self, dir: Direction) -> bool {
        let wrap_back = self.focused == 0 && dir == Backward;
        let wrap_forward = self.focused == self.inner.len() - 1 && dir == Forward;

        wrap_back || wrap_forward
    }

    pub fn next_index(&self, direction: Direction) -> usize {
        let max = self.inner.len() - 1;

        match direction {
            Forward => {
                if self.focused == max {
                    0
                } else {
                    self.focused + 1
                }
            }
            Backward => {
                if self.focused == 0 {
                    max
                } else {
                    self.focused - 1
                }
            }
        }
    }

    #[inline]
    pub fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(&T) -> bool,
    {
        self.inner.iter().position(predicate)
    }

    /// Focus the first element matching the given predicate.
    ///
    /// If an element is found this method returns whether or not this induces a focus change and
    /// the index of the new focus point, otherwise None.
    pub fn focus<P>(&mut self, predicate: P) -> Result<(bool, usize)>
    where
        P: Fn(&T) -> bool,
    {
        match self.position(predicate) {
            Some(index) if index == self.focused => Ok((false, index)),
            Some(index) => {
                self.focused = index;
                Ok((true, index))
            }
            None => Err(Error::NoMatchingElement),
        }
    }

    #[inline]
    pub fn cycle_focus(&mut self, direction: Direction) -> Option<&T> {
        self.focused = self.next_index(direction);
        self.focused_element()
    }

    pub fn drag_focused(&mut self, direction: Direction) -> Option<&T> {
        if self.inner.is_empty() {
            return None;
        }

        match (self.focused, self.next_index(direction), direction) {
            (0, _, Backward) | (_, 0, Forward) => self.rotate(direction),
            (focused, other, _) => self.inner.swap(focused, other),
        }

        self.cycle_focus(direction)
    }

    #[inline]
    fn clamp_focus(&mut self) {
        if self.focused > 0 && self.focused >= self.inner.len() - 1 {
            self.focused -= 1;
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T> {
        let t = self.inner.remove(index);
        self.clamp_focus();
        t
    }

    #[inline]
    pub fn remove_focused(&mut self) -> Option<T> {
        let t = self.inner.remove(self.focused);
        self.clamp_focus();
        t
    }

    #[inline]
    pub fn swap(&mut self, i: usize, j: usize) {
        self.inner.swap(i, j);
    }

    #[inline]
    pub fn swap_focused_with_front(&mut self) {
        if !self.inner.is_empty() {
            self.inner.swap(0, self.focused);
            self.focused = 0;
        }
    }
}

impl<T: PartialEq> Ring<T> {
    #[inline]
    pub fn contains(&self, t: &T) -> bool {
        self.inner.contains(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn rotate_empty() {
        let mut r: Ring<i32> = Ring::default();
        r.rotate(Backward);

        assert!(r.elements().is_empty());
    }

    #[test_case(Forward, &[3, 1, 2]; "forward")]
    #[test_case(Backward, &[2, 3, 1]; "backward")]
    fn rotate(d: Direction, expected: &[i32]) {
        let mut r = Ring::from(vec![1, 2, 3]);
        r.rotate(d);

        assert_eq!(r.elements(), expected);
    }

    #[test_case(Forward, Backward; "forward then backward")]
    #[test_case(Backward, Forward; "backward then forward")]
    fn rotate_forward_backward_are_inverse(first: Direction, second: Direction) {
        let mut r = Ring::from(vec![1, 2, 3]);
        r.rotate(first);
        r.rotate(second);

        assert_eq!(r.elements(), &[1, 2, 3]);
    }

    #[test_case(Forward, &[3, 1, 2], 3; "forward")]
    #[test_case(Backward, &[2, 3, 1], 2; "backward")]
    fn rotate_holds_focused_index_constant(d: Direction, expected: &[i32], focused: i32) {
        let mut r = Ring::from(vec![1, 2, 3]);
        assert_eq!(r.focused_element(), Some(&1));

        r.rotate(d);
        assert_eq!(r.elements(), expected);
        assert_eq!(r.focused, 0);
        assert_eq!(r.focused_element(), Some(&focused));
    }

    #[test_case(Forward, 0, 1; "forward")]
    #[test_case(Forward, 3, 0; "forward wrapping")]
    #[test_case(Backward, 1, 0; "backward")]
    #[test_case(Backward, 0, 3; "backward wrapping")]
    fn cycle_focus(d: Direction, initial: usize, expected: usize) {
        let mut r = Ring::from(vec![1, 2, 3, 4]);
        r.focused = initial;
        r.cycle_focus(d);

        assert_eq!(r.focused, expected);
    }

    #[test_case(Forward, &[&[2, 1, 3], &[2, 3, 1]]; "forward")]
    #[test_case(Backward, &[&[2, 3, 1], &[2, 1, 3]]; "backward")]
    fn drag_focused(d: Direction, expected: &[&[i32]]) {
        let mut r = Ring::from(vec![1, 2, 3]);
        assert_eq!(r.focused_element(), Some(&1));

        assert_eq!(r.drag_focused(d), Some(&1));
        assert_eq!(r.elements(), expected[0]);

        assert_eq!(r.drag_focused(d), Some(&1));
        assert_eq!(r.elements(), expected[1]);

        assert_eq!(r.drag_focused(d), Some(&1));
        assert_eq!(r.elements(), &[1, 2, 3]);
    }

    #[test_case(0, 1, Some((false, 0)); "current focused")]
    #[test_case(0, 2, Some((true, 1)); "new focus")]
    #[test_case(0, 42, None; "not found")]
    fn focus(focused: usize, target: i32, expected: Option<(bool, usize)>) {
        let mut r = Ring::from(vec![1, 2, 3, 4]);
        r.focused = focused;

        if let Some((_, should_be_focus)) = expected {
            assert_eq!(r.focus(|&e| e == target).ok(), expected);
            assert_eq!(r.focused, should_be_focus);
        } else {
            assert!(matches!(
                r.focus(|&e| e == target),
                Err(Error::NoMatchingElement)
            ));
        }
    }

    #[test_case(1, 0, Some(1), 3; "before focus point")]
    #[test_case(1, 1, Some(2), 3; "focus point")]
    #[test_case(0, 1, Some(2), 1; "after focus point")]
    #[test_case(3, 3, Some(4), 3; "last when focused should clamp")]
    fn remove(focused: usize, index: usize, expected: Option<i32>, focus_after: i32) {
        let mut r = Ring::from(vec![1, 2, 3, 4]);
        r.focused = focused;

        assert_eq!(r.remove(index), expected);
        assert_eq!(r.focused_element(), Some(&focus_after));
    }

    #[test]
    fn repeated_remove_focused() {
        let mut r = Ring::from(vec![1, 2, 3]);
        r.focused = 2;

        assert_eq!(r.remove_focused(), Some(3));
        assert_eq!(r.focused, 1);

        assert_eq!(r.remove_focused(), Some(2));
        assert_eq!(r.focused, 0);

        assert_eq!(r.remove_focused(), Some(1));
        assert_eq!(r.focused, 0);

        assert_eq!(r.remove_focused(), None);
        assert_eq!(r.focused, 0);
    }

    #[test_case(2, Some(1); "it works")]
    #[test_case(3, Some(2); "first matching is returned")]
    #[test_case(42, None; "no match returns None")]
    fn position(n: i32, expected: Option<usize>) {
        let r = Ring::from(vec![1, 2, 3, 3, 4, 5]);

        assert_eq!(r.position(|&k| k == n), expected);
    }

    #[test_case(First, &[42, 1, 2, 3, 4]; "first")]
    #[test_case(Last, &[1, 2, 3, 4, 42]; "last")]
    #[test_case(Index(3), &[1, 2, 3, 42, 4]; "index")]
    #[test_case(Focused, &[1, 42, 2, 3, 4]; "focused")]
    #[test_case(AfterFocused, &[1, 2, 42, 3, 4]; "after focused")]
    fn insert(ip: InsertPoint, expected: &[i32]) {
        let mut r = Ring::from(vec![1, 2, 3, 4]);
        r.focused = 1;

        r.insert(42, ip);

        assert_eq!(r.elements(), expected);
    }

    #[test_case(0, vec![1, 2, 3, 4], &[1, 2, 3, 4]; "valid front")]
    #[test_case(2, vec![1, 2, 3, 4], &[3, 2, 1, 4]; "valid not front")]
    #[test_case(0, vec![], &[]; "empty")]
    fn swap_focused_with_front(focused: usize, inner: Vec<i32>, expected: &[i32]) {
        let mut r = Ring::from(inner);
        r.focused = focused;

        r.swap_focused_with_front();

        assert_eq!(r.elements(), expected);
        assert_eq!(r.focused, 0);
    }
}
