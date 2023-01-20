use core::marker::PhantomData;
use fn_traits::FnMut;

pub struct ErrFn<T> {
    phantom: PhantomData<fn() -> T>,
}

impl<T> Default for ErrFn<T> {
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<T> Clone for ErrFn<T> {
    fn clone(&self) -> Self {
        Self { phantom: self.phantom }
    }
}

impl<T, E> FnMut<(E,)> for ErrFn<T> {
    type Output = Result<T, E>;

    fn call_mut(&mut self, args: (E,)) -> Self::Output {
        Err(args.0)
    }
}
