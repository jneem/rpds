/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cmp::Ordering;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::sync::Arc;
use std::marker::PhantomData;
use utils::container::Container;

// TODO Use impl trait instead of this when available.
pub type Iter<'a, T, DataContainer> =
    ::std::iter::Map<IterContainer<'a, T, DataContainer>, fn(&DataContainer) -> &T>;

#[doc(hidden)]
#[macro_export]
macro_rules! list_reverse {
    ( ; $($reversed:expr),*) => {
         {
            #[allow(unused_mut)]
            let mut l = $crate::List::new();
            $(
                l.push_front_mut($reversed);
            )*
            l
        }
    };
    ($h:expr ; $($reversed:expr),*) => {
        list_reverse!( ; $h, $($reversed),*)
    };
    ($h:expr, $($t:expr),+ ; $($reversed:expr),*) => {
        list_reverse!($($t),* ; $h, $($reversed),*)
    };

    // This is just to handle the cases where this macro is called with an extra comma in the
    // reserve list, which can happen in a recursive call.
    ($($t:expr),* ; $($reversed:expr),*,) => {
        list_reverse!($($t),* ; $($reversed),*)
    };
}

/// Creates a [`List`](list/struct.List.html) containing the given arguments:
///
/// ```
/// # use rpds::*;
/// #
/// let l = List::new()
///     .push_front(3)
///     .push_front(2)
///     .push_front(1);
///
/// assert_eq!(list![1, 2, 3], l);
/// ```
#[macro_export]
macro_rules! list {
    ($($e:expr),*) => {
        list_reverse!($($e),* ; )
    };
}

/// A persistent list with structural sharing.
///
/// # Complexity
///
/// Let *n* be the number of elements in the list.
///
/// ## Temporal complexity
///
/// | Operation         | Best case | Average | Worst case  |
/// |:----------------- | ---------:| -------:| -----------:|
/// | `new()`           |      Θ(1) |    Θ(1) |        Θ(1) |
/// | `push_front()`    |      Θ(1) |    Θ(1) |        Θ(1) |
/// | `drop_first()`    |      Θ(1) |    Θ(1) |        Θ(1) |
/// | `reverse()`       |      Θ(n) |    Θ(n) |        Θ(n) |
/// | `first()`         |      Θ(1) |    Θ(1) |        Θ(1) |
/// | `last()`          |      Θ(1) |    Θ(1) |        Θ(1) |
/// | `len()`           |      Θ(1) |    Θ(1) |        Θ(1) |
/// | `clone()`         |      Θ(1) |    Θ(1) |        Θ(1) |
/// | iterator creation |      Θ(1) |    Θ(1) |        Θ(1) |
/// | iterator step     |      Θ(1) |    Θ(1) |        Θ(1) |
/// | iterator full     |      Θ(n) |    Θ(n) |        Θ(n) |
///
/// # Implementation details
///
/// This is your classic functional list with "cons" and "nil" nodes, with a little extra sauce to
/// make some operations more efficient.
#[derive(Debug)]
// WIP pub struct List<T, DataContainer = Rc<T>>
pub struct List<T, DataContainer>
    where DataContainer: Container<T>
{
    head:   Option<Arc<Node<T, DataContainer>>>,
    last:   Option<DataContainer>,
    length: usize,
}

#[derive(Debug)]
struct Node<T, DataContainer>
    where DataContainer: Container<T>
{
    value: DataContainer,
    next:  Option<Arc<Node<T, DataContainer>>>,
    phantom_t: PhantomData<T>
}

impl<T, DataContainer> Clone for Node<T, DataContainer>
    where DataContainer: Container<T>
{
    fn clone(&self) -> Node<T, DataContainer> {
        Node {
            value: DataContainer::clone(&self.value),
            next:  self.next.clone(),
            phantom_t: PhantomData
        }
    }
}

impl<T, DataContainer> List<T, DataContainer>
    where DataContainer: Container<T>
{
    #[must_use]
    pub fn new() -> List<T, DataContainer> {
        List {
            head:   None,
            last:   None,
            length: 0,
        }
    }

    #[must_use]
    pub fn first(&self) -> Option<&T> {
        self.head.as_ref().map(|node| node.value.borrow())
    }

    #[must_use]
    pub fn last(&self) -> Option<&T> {
        self.last.as_ref().map(|node| node.borrow())
    }

    #[must_use]
    pub fn drop_first(&self) -> Option<List<T, DataContainer>> {
        let mut new_list = self.clone();

        if new_list.drop_first_mut() {
            Some(new_list)
        } else {
            None
        }
    }

    pub fn drop_first_mut(&mut self) -> bool {
        if let Some(ref h) = self.head.take() {
            self.head = h.next.clone();
            self.length -= 1;

            if self.length == 0 {
                self.last = None;
            }

            true
        } else {
            false
        }
    }

    fn push_front_arc_mut(&mut self, v: DataContainer) {
        if self.length == 0 {
            self.last = Some(DataContainer::clone(&v));
        }

        let new_head = Node {
            value: v,
            next:  self.head.take(),
            phantom_t: PhantomData
        };

        self.head = Some(Arc::new(new_head));
        self.length += 1;
    }

    #[must_use]
    pub fn push_front(&self, v: T) -> List<T, DataContainer> {
        let mut new_list = self.clone();

        new_list.push_front_mut(v);

        new_list
    }

    pub fn push_front_mut(&mut self, v: T) {
        self.push_front_arc_mut(DataContainer::new(v))
    }

    #[must_use]
    pub fn reverse(&self) -> List<T, DataContainer> {
        let mut new_list = List::new();

        // It is significantly faster to re-implement this here than to clone and call
        // `reverse_mut()`.  The reason is that since this is a linear data structure all nodes will
        // need to be cloned given that the ref count would be greater than one.

        for v in self.iter_container() {
            new_list.push_front_arc_mut(DataContainer::clone(&v));
        }

        new_list
    }

    pub fn reverse_mut(&mut self) {
        self.last = self.head.as_ref().map(|next| DataContainer::clone(&next.value));

        let mut prev: Option<Arc<Node<T, DataContainer>>> = None;
        let mut current: Option<Arc<Node<T, DataContainer>>> = self.head.take();

        while let Some(mut curr_arc) = current {
            // TODO Simplify once we have NLL.
            {
                let curr = Arc::make_mut(&mut curr_arc);
                let curr_next = curr.next.take();

                curr.next = prev.take();

                current = curr_next;
            }
            prev = Some(curr_arc);
        }

        self.head = prev;
    }

    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn iter(&self) -> Iter<T, DataContainer> {
        self.iter_container().map(|v| v.borrow())
    }

    pub(crate) fn iter_container(&self) -> IterContainer<T, DataContainer> {
        IterContainer::new(self)
    }
}

impl<T, DataContainer> Default for List<T, DataContainer>
    where DataContainer: Container<T>
{
    fn default() -> List<T, DataContainer> {
        List::new()
    }
}

impl<T: PartialEq, DataContainer, DataContainerOther> PartialEq<List<T, DataContainerOther>> for List<T, DataContainer>
    where DataContainer: Container<T>,
          DataContainerOther: Container<T>
{
    fn eq(&self, other: &List<T, DataContainerOther>) -> bool {
        self.length == other.length && self.iter().eq(other.iter())
    }
}

impl<T: Eq, DataContainer> Eq for List<T, DataContainer>
    where DataContainer: Container<T>
{}

// WIP utest to make sure it plays together with container of different types.
impl<T: PartialOrd<T>, DataContainer, DataContainerOther> PartialOrd<List<T, DataContainerOther>> for List<T, DataContainer>
    where DataContainer: Container<T>,
          DataContainerOther: Container<T>
{
    fn partial_cmp(&self, other: &List<T, DataContainerOther>) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: Ord, DataContainer> Ord for List<T, DataContainer>
    where DataContainer: Container<T>
{
    fn cmp(&self, other: &List<T, DataContainer>) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T: Hash, DataContainer> Hash for List<T, DataContainer>
    where DataContainer: Container<T>
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Add the hash of length so that if two collections are added one after the other it doesn't
        // hash to the same thing as a single collection with the same elements in the same order.
        self.len().hash(state);

        for e in self {
            e.hash(state);
        }
    }
}

impl<T, DataContainer> Clone for List<T, DataContainer>
    where DataContainer: Container<T>
{
    fn clone(&self) -> List<T, DataContainer> {
        List {
            head:   self.head.clone(),
            last:   self.last.clone(),
            length: self.length,
        }
    }
}

impl<T: Display, DataContainer> Display for List<T, DataContainer>
    where DataContainer: Container<T>
{
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let mut first = true;

        fmt.write_str("[")?;

        for v in self.iter() {
            if !first {
                fmt.write_str(", ")?;
            }
            v.fmt(fmt)?;
            first = false;
        }

        fmt.write_str("]")
    }
}

impl<'a, T, DataContainer> IntoIterator for &'a List<T, DataContainer>
    where DataContainer: Container<T>
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T, DataContainer>;

    fn into_iter(self) -> Iter<'a, T, DataContainer> {
        self.iter()
    }
}

impl<T, DataContainer> FromIterator<T> for List<T, DataContainer>
    where DataContainer: Container<T>
{
    fn from_iter<I: IntoIterator<Item = T>>(into_iter: I) -> List<T, DataContainer> {
        let iter = into_iter.into_iter();
        let (min_size, max_size_hint) = iter.size_hint();
        let mut vec: Vec<T> = Vec::with_capacity(max_size_hint.unwrap_or(min_size));

        for e in iter {
            vec.push(e);
        }

        let mut list = List::new();

        for e in vec.into_iter().rev() {
            list.push_front_mut(e);
        }

        list
    }
}

#[derive(Debug)]
pub struct IterContainer<'a, T: 'a, DataContainer: 'a>
    where DataContainer: Container<T>
{
    next:   Option<&'a Node<T, DataContainer>>,
    length: usize,
}

impl<'a, T, DataContainer> IterContainer<'a, T, DataContainer>
    where DataContainer: Container<T>
{
    fn new(list: &List<T, DataContainer>) -> IterContainer<T, DataContainer> {
        IterContainer {
            next:   list.head.as_ref().map(|node| node.as_ref()),
            length: list.len(),
        }
    }
}

impl<'a, T, DataContainer> Iterator for IterContainer<'a, T, DataContainer>
    where DataContainer: Container<T>
{
    type Item = &'a DataContainer;

    fn next(&mut self) -> Option<&'a DataContainer> {
        match self.next {
            Some(&Node {
                value: ref v,
                next: ref t,
                ..
            }) => {
                self.next = t.as_ref().map(|node| node.as_ref());
                self.length -= 1;
                Some(v)
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.length, Some(self.length))
    }
}

impl<'a, T, DataContainer> ExactSizeIterator for IterContainer<'a, T, DataContainer>
    where DataContainer: Container<T>
{}

#[cfg(feature = "serde")]
pub mod serde {
    use super::*;
    use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
    use serde::ser::{Serialize, Serializer};
    use std::fmt;
    use std::marker::PhantomData;

    impl<T, DataContainer> Serialize for List<T, DataContainer>
    where
        T: Serialize,
        DataContainer: Container<T>
    {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_seq(self)
        }
    }

    impl<'de, T, DataContainer> Deserialize<'de> for List<T, DataContainer>
    where
        T: Deserialize<'de>,
        DataContainer: Container<T>
    {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<List<T, DataContainer>, D::Error> {
            deserializer.deserialize_seq(ListVisitor {
                phantom_t: PhantomData,
                phantom_data_container: PhantomData
            })
        }
    }

    struct ListVisitor<T, DataContainer>
        where DataContainer: Container<T>
     {
        phantom_t: PhantomData<T>,
        phantom_data_container: PhantomData<DataContainer>,
    }

    impl<'de, T, DataContainer> Visitor<'de> for ListVisitor<T, DataContainer>
    where
        T: Deserialize<'de>,
        DataContainer: Container<T>
    {
        type Value = List<T, DataContainer>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<List<T, DataContainer>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec: Vec<T> = if let Some(capacity) = seq.size_hint() {
                Vec::with_capacity(capacity)
            } else {
                Vec::new()
            };

            while let Some(value) = seq.next_element()? {
                vec.push(value);
            }

            let mut list: List<T, DataContainer> = List::new();

            for value in vec.into_iter().rev() {
                list.push_front_mut(value);
            }

            Ok(list)
        }
    }
}

/*
#[cfg(test)]
mod test;
*/