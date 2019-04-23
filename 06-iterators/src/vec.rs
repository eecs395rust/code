use std::{cmp, mem};
use super::iter8or::*;

/// First, a Vector iterator. We're make a read-only, by-reference
/// iterator, which is the default (and the only one we can do without
/// special knowledge of `Vec`'s implementation. So we store a reference
/// to a vector and the position of the next element to return.
pub struct VecIter<'a, T: 'a> {
    base: &'a Vec<T>,
    pos:  usize,
}

impl<'a, T> Iter8or for VecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.pos < self.base.len() {
            let result = &self.base[self.pos];
            self.pos += 1;
            Some(result)
        } else {None}
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Uses `ExactSizeIter8or` from below.
        (self.len(), Some(self.len()))
    }
}

impl<'a, T> ExactSizeIter8or for VecIter<'a, T> {
    fn len(&self) -> usize {
        self.base.len() - self.pos
    }
}

impl<T> FromIter8or<T> for Vec<T> {
    fn from_iter<I: IntoIter8or<Item = T>>(pre_iter: I) -> Self {
        let mut iter = pre_iter.into_iter8or();

        let mut result = {
            let (lower, upper_option) = iter.size_hint();
            let capacity = match upper_option {
                Some(upper) => cmp::min(2 * lower, upper),
                None => lower,
            };
            Vec::with_capacity(capacity)
        };

        while let Some(item) = iter.next() {
            result.push(item)
        }

        result
    }
}

/// What if we want to implement `DoubleEndedIter8or` for `VecIter`?
/// Well, we would have to add another field. But wait a minute.
/// Remember how a reference to a vector isn't usually a useful type,
/// and we'd usually use a slice instead? Well, a slice already supports
/// double-ended iteration! Watch:
pub struct SliceIter<'a, T: 'a>(&'a [T]);

impl<'a, T> Iter8or for SliceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.0.split_first().map(|(result, rest)| {
            self.0 = rest;
            result
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T> IntoIter8or for &'a [T] {
    type Item = &'a T;
    type IntoIter = SliceIter<'a, T>;

    fn into_iter8or(self) -> SliceIter<'a, T> {
        SliceIter(self)
    }
}

impl<'a, T> IntoIter8or for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = SliceIter<'a, T>;

    fn into_iter8or(self) -> SliceIter<'a, T> {
        SliceIter(self.as_slice())
    }
}

impl<'a, T> ExactSizeIter8or for SliceIter<'a, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T> DoubleEndedIter8or for SliceIter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.0.split_last().map(|(result, rest)| {
            self.0 = rest;
            result
        })
    }
}

pub struct SliceIterMut<'a, T: 'a>(&'a mut [T]);

impl<'a, T> Iter8or for SliceIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        let tmp = mem::replace(&mut self.0, &mut []);
        tmp.split_first_mut().map(|(first, rest)| {
            self.0 = rest;
            first
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T> IntoIter8or for &'a mut Vec<T> {
    type Item = &'a mut T;
    type IntoIter = SliceIterMut<'a, T>;

    fn into_iter8or(self) -> <Self as IntoIter8or>::IntoIter {
        SliceIterMut(self.as_mut_slice())
    }
}

impl<'a, T> IntoIter8or for &'a mut [T] {
    type Item = &'a mut T;
    type IntoIter = SliceIterMut<'a, T>;

    fn into_iter8or(self) -> <Self as IntoIter8or>::IntoIter {
        SliceIterMut(self)
    }
}

impl<'a, T> ExactSizeIter8or for SliceIterMut<'a, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T> DoubleEndedIter8or for SliceIterMut<'a, T> {
    fn next_back(&mut self) -> Option<<Self as Iter8or>::Item> {
        let tmp = mem::replace(&mut self.0, &mut []);
        tmp.split_last_mut().map(|(last, rest)| {
            self.0 = rest;
            last
        })
    }
}

// The real `std::vec::IntoIter` is constructed from a `Vec` in
// constant (perhaps 0) time. This is stored in reverse, so
// constructing requires linear time. The linear time is amortized
// over the cost of iterating over the elements.
pub struct VecIntoIter<T>(Vec<T>);

impl<T> Iter8or for VecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<<Self as Iter8or>::Item> {
        self.0.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.len();
        (len, Some(len))
    }
}

impl<T> ExactSizeIter8or for VecIntoIter<T> {}

impl<T> IntoIter8or for Vec<T> {
    type Item = T;
    type IntoIter = VecIntoIter<T>;

    fn into_iter8or(mut self) -> <Self as IntoIter8or>::IntoIter {
        let mut result = VecIntoIter(Vec::with_capacity(self.len()));

        while let Some(item) = self.pop() {
            result.0.push(item);
        }

        result
    }
}
