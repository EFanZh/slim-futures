use crate::support::fn_mut_1::FnMut1;
use std::marker::PhantomData;

pub struct OkFn<T, E> {
    _phantom: PhantomData<fn(T) -> Result<T, E>>,
}

impl<T, E> Default for OkFn<T, E> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<T, E> Clone for OkFn<T, E> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<T, E> FnMut1<T> for OkFn<T, E> {
    type Output = Result<T, E>;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        Ok(arg)
    }
}
