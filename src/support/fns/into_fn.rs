use core::marker::PhantomData;
use fn_traits::FnMut;

pub struct IntoFn<T, U> {
    phantom: PhantomData<fn(T) -> U>,
}

impl<T, U> Default for IntoFn<T, U> {
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<T, U> Clone for IntoFn<T, U> {
    fn clone(&self) -> Self {
        Self { phantom: self.phantom }
    }
}

impl<T, U> FnMut<(T,)> for IntoFn<T, U>
where
    T: Into<U>,
{
    type Output = U;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        args.0.into()
    }
}
