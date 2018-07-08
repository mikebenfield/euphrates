use std::ops::{Deref, DerefMut};

/// Can `Deref` to a `&T`, either by owning a `T` or having a reference to one.
pub enum Cref<'a, T: 'a> {
    Const(&'a T),
    Own(T),
}

impl<'a, T: 'a> Deref for Cref<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            Cref::Const(ref r) => &*r,
            Cref::Own(ref t) => t,
        }
    }
}

/// Can `DerefMut` to a `&mut T`, either by owning a `T` or having a mutable
/// reference to one.
pub enum Mref<'a, T: 'a> {
    Mut(&'a mut T),
    Own(T),
}

impl<'a, T: 'a> Deref for Mref<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        match self {
            Mref::Mut(ref r) => &*r,
            Mref::Own(ref t) => t,
        }
    }
}

impl<'a, T: 'a> DerefMut for Mref<'a, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        match self {
            Mref::Mut(ref mut r) => &mut *r,
            Mref::Own(ref mut t) => t,
        }
    }
}

enum RefE<T: ?Sized> {
    Const(*const T),
    Mut(*mut T),
}

/// A constant or mutable reference unprotected by lifetimes.
///
/// `new` and `new_mut` are marked `unsafe` because it is considered the
/// obligation of the caller of these functions to immediately put the `Ref`
/// into a `Mref` or `Cref`, thus allowing the borrow checker to do its job.
pub struct Ref<T: ?Sized>(RefE<T>);

impl<T: ?Sized> Ref<T> {
    #[inline(always)]
    pub unsafe fn new(t: &T) -> Self {
        Ref(RefE::Const(t))
    }

    #[inline(always)]
    pub unsafe fn new_mut(t: &mut T) -> Self {
        Ref(RefE::Mut(t))
    }

    /// Get a reference to the `T`.
    #[inline(always)]
    pub fn _0(&self) -> &T {
        match self.0 {
            RefE::Const(ref r) => unsafe { &**r },
            RefE::Mut(ref r) => unsafe { &**r },
        }
    }

    /// Get a mutable reference to the `T`.
    ///
    /// Panics if the `Ref` was created with `new` instead of `new_mut`.
    #[inline(always)]
    pub fn mut_0(&mut self) -> &mut T {
        match self.0 {
            RefE::Mut(ref mut r) => unsafe { &mut **r },
            _ => panic!("Ref"),
        }
    }
}

pub trait Impl<I: ?Sized> {
    type Impler;
    fn make<'a>(&'a self) -> Cref<'a, Self::Impler>;
    fn make_mut<'a>(&'a mut self) -> Mref<'a, Self::Impler>;
}
