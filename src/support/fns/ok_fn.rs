use core::marker::PhantomData;
use fn_traits::FnMut;

pub struct OkFn<E> {
    phantom: PhantomData<fn() -> E>,
}

impl<E> Default for OkFn<E> {
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<E> Clone for OkFn<E> {
    fn clone(&self) -> Self {
        Self { phantom: self.phantom }
    }
}

impl<T, E> FnMut<(T,)> for OkFn<E> {
    type Output = Result<T, E>;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        Ok(args.0)
    }
}
