use crate::support::fn_mut_1::FnMut1;
use std::marker::PhantomData;

pub struct ErrFn<T, E> {
    _phantom: PhantomData<fn() -> Result<T, E>>,
}

impl<T, E> Default for ErrFn<T, E> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<T, E> Clone for ErrFn<T, E> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<T, E> FnMut1<E> for ErrFn<T, E> {
    type Output = Result<T, E>;

    fn call_mut(&mut self, arg: E) -> Self::Output {
        Err(arg)
    }
}
