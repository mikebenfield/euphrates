/// Unsafe references.
pub enum ConstOrMut<T: ?Sized> {
    Const(*const T),
    Mut(*mut T),
}

impl<T: ?Sized> ConstOrMut<T> {
    #[inline]
    pub unsafe fn _0(&self) -> &T {
        match self {
            &ConstOrMut::Mut(ref t) => &**t,
            &ConstOrMut::Const(ref t) => &**t,
        }
    }

    #[inline]
    pub unsafe fn mut_0(&mut self) -> &mut T {
        match self {
            &mut ConstOrMut::Mut(ref mut t) => &mut **t,
            _ => panic!("ConstOrMut"),
        }
    }
}

pub trait Impler {
    type T: ?Sized;

    fn iclose<F, U>(t: &Self::T, f: F) -> U
    where
        F: FnOnce(&Self) -> U;

    fn iclose_mut<F, U>(t: &mut Self::T, f: F) -> U
    where
        F: FnOnce(&mut Self) -> U;

    fn _0(&self) -> &Self::T;

    fn mut_0(&mut self) -> &mut Self::T;
}

pub unsafe trait ImplerImpl {
    type T: ?Sized;

    unsafe fn new(c: ConstOrMut<Self::T>) -> Self;

    fn get(&self) -> &ConstOrMut<Self::T>;

    fn get_mut(&mut self) -> &mut ConstOrMut<Self::T>;
}

impl<V> Impler for V
where
    V: ImplerImpl,
{
    type T = V::T;

    #[inline]
    fn iclose<F, U>(t: &Self::T, f: F) -> U
    where
        F: FnOnce(&Self) -> U,
    {
        let v = unsafe { Self::new(ConstOrMut::Const(t)) };
        f(&v)
    }

    #[inline]
    fn iclose_mut<F, U>(t: &mut Self::T, f: F) -> U
    where
        F: FnOnce(&mut Self) -> U,
    {
        let mut v = unsafe { Self::new(ConstOrMut::Mut(t)) };
        f(&mut v)
    }

    #[inline]
    fn _0(&self) -> &Self::T {
        unsafe { self.get()._0() }
    }

    #[inline]
    fn mut_0(&mut self) -> &mut Self::T {
        unsafe { self.get_mut().mut_0() }
    }
}
