use core::marker::PhantomData;
use fn_traits::FnMut;

pub struct OkFn<T, E> {
    phantom: PhantomData<fn(T) -> Result<T, E>>,
}

impl<T, E> Default for OkFn<T, E> {
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<T, E> Clone for OkFn<T, E> {
    fn clone(&self) -> Self {
        Self { phantom: self.phantom }
    }
}

impl<T, E> FnMut<(T,)> for OkFn<T, E> {
    type Output = Result<T, E>;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        Ok(args.0)
    }
}
