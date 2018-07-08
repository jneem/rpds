/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
pub struct Owned<T: Clone>(T);

pub trait Container<T>: Deref<Target = T> + Clone + Borrow<T> {
    fn new(v: T) -> Self;
}

impl<T> Container<T> for Arc<T> {
    fn new(v: T) -> Arc<T> {
        Arc::new(v)
    }
}

impl<T> Container<T> for Rc<T> {
    fn new(v: T) -> Rc<T> {
        Rc::new(v)
    }
}

impl<T: Clone> Deref for Owned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let Owned(v) = self;

        &v
    }
}

impl<T: Clone> DerefMut for Owned<T> {
    fn deref_mut(&mut self) -> &mut T {
        let Owned(ref mut v) = self;

        v
    }
}

impl<T: Clone> Container<T> for Owned<T> {
    fn new(v: T) -> Owned<T> {
        Owned(v)
    }
}

impl<T: Clone> Borrow<T> for Owned<T> {
    fn borrow(&self) -> &T {
        self.deref()
    }
}
