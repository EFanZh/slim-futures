use core::marker::PhantomData;
use fn_traits::FnMut;

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

impl<T, E> FnMut<(E,)> for ErrFn<T, E> {
    type Output = Result<T, E>;

    fn call_mut(&mut self, args: (E,)) -> Self::Output {
        Err(args.0)
    }
}
