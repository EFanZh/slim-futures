use core::marker::PhantomData;
use fn_traits::FnMut;

pub struct IntoFn<U> {
    phantom: PhantomData<fn() -> U>,
}

impl<U> Default for IntoFn<U> {
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<U> Clone for IntoFn<U> {
    fn clone(&self) -> Self {
        Self { phantom: self.phantom }
    }
}

impl<T, U> FnMut<(T,)> for IntoFn<U>
where
    T: Into<U>,
{
    type Output = U;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        args.0.into()
    }
}
