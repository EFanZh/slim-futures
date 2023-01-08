use crate::support::fn_mut_1::FnMut1;
use core::marker::PhantomData;

pub struct IntoFn<T, U> {
    _phantom: PhantomData<fn(T) -> U>,
}

impl<T, U> Default for IntoFn<T, U> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<T, U> Clone for IntoFn<T, U> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<T, U> FnMut1<T> for IntoFn<T, U>
where
    T: Into<U>,
{
    type Output = U;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        arg.into()
    }
}
