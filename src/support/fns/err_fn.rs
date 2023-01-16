use crate::support::fn_mut_1::FnMut1;
use core::marker::PhantomData;

pub struct ErrFn<T, E> {
    phantom: PhantomData<fn(E) -> Result<T, E>>,
}

impl<T, E> Default for ErrFn<T, E> {
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<T, E> Clone for ErrFn<T, E> {
    fn clone(&self) -> Self {
        Self { phantom: self.phantom }
    }
}

impl<T, E> FnMut1<E> for ErrFn<T, E> {
    type Output = Result<T, E>;

    fn call_mut(&mut self, arg: E) -> Self::Output {
        Err(arg)
    }
}
