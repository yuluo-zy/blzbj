use std::marker::PhantomData;

pub struct Skip;
pub struct Take;

pub struct Handle<T, N> {
    phantom: PhantomData<(T, N)>,
}
pub(crate) fn new_handle<T, N>() -> Handle<T, N> {
    Handle {
        phantom: PhantomData,
    }
}

impl<T, N> Clone for Handle<T, N> {
    fn clone(&self) -> Handle<T, N> {
        *self
    }
}

// Derived `Copy` doesn't work here.
impl<T, N> Copy for Handle<T, N> {}

pub trait Lookup<T, N> {
    #[doc(hidden)]
    fn get_from(&self) -> &T;
}

#[doc(hidden)]
impl<T, U, V, N> Lookup<T, (Skip, N)> for (U, V)
    where
        V: Lookup<T, N>,
{
    fn get_from(&self) -> &T {
        self.1.get_from()
    }
}

#[doc(hidden)]
impl<T, V> Lookup<T, Take> for (T, V) {
    fn get_from(&self) -> &T {
        &self.0
    }
}


pub trait Append<T> {
    /// The resulting `BorrowBag` type parameter after adding an element of type `T`.
    type Output: PrefixedWith<Self> + Lookup<T, Self::Navigator>;

    /// A type describing how to borrow the `T` which is added.
    ///
    /// If the output type is `(X, (Y, (Z, ())))`, we're adding the `Z` and so our `Navigator` will
    /// be `(Skip, (Skip, Take))`
    type Navigator;

    /// Append the element, returning a new collection and a handle to borrow the element back.
    fn append(self, t: T) -> (Self::Output, Handle<T, Self::Navigator>);
}

impl<T, U, V> Append<T> for (U, V)
    where
        V: Append<T>,
{
    // We're mid-list. Return the head and append to the tail.
    type Output = (U, <V as Append<T>>::Output);

    // We're mid-list. Skip this element and navigate again in the tail.
    type Navigator = (Skip, <V as Append<T>>::Navigator);

    fn append(self, t: T) -> (Self::Output, Handle<T, Self::Navigator>) {
        let (u, v) = self;
        ((u, v.append(t).0), new_handle())
    }
}

impl<T> Append<T> for () {
    // This is the end of the added elements. We insert our `T` before the end.
    type Output = (T, ());

    // We're adding our `T` here, so we take this element on navigation.
    type Navigator = Take;

    fn append(self, t: T) -> (Self::Output, Handle<T, Self::Navigator>) {
        ((t, ()), new_handle())
    }
}

/// Provides proof that the existing list elements don't move, which guarantees that existing
/// `Handle` values continue to work.
pub trait PrefixedWith<T>
    where
        T: ?Sized,
{
}

impl<U, V0, V1> PrefixedWith<(U, V0)> for (U, V1) where V1: PrefixedWith<V0> {}
impl<U> PrefixedWith<()> for (U, ()) {}

#[derive(Default)]
pub struct BorrowBag<V> {
    v: V,
}

impl BorrowBag<()> {
    /// Creates a new, empty `BorrowBag`.
    pub fn new() -> Self {
        BorrowBag { v: () }
    }
}

impl<V> BorrowBag<V> {

    pub fn add<T>(self, t: T) -> (BorrowBag<V::Output>, Handle<T, V::Navigator>)
        where
            V: Append<T>,
    {
        let (v, handle) = Append::append(self.v, t);
        (BorrowBag { v }, handle)
    }


    pub fn borrow<T, N>(&self, _handle: Handle<T, N>) -> &T
        where
            V: Lookup<T, N>,
    {
        Lookup::<T, N>::get_from(&self.v)
    }
}


