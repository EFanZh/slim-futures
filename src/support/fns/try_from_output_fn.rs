use crate::support::try_::Try;
use core::marker::PhantomData;
use fn_traits::FnMut;

pub struct TryFromOutputFn<T> {
    phantom: PhantomData<T>,
}

impl<T> Default for TryFromOutputFn<T> {
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<T> Clone for TryFromOutputFn<T> {
    fn clone(&self) -> Self {
        Self { phantom: self.phantom }
    }
}

impl<T> FnMut<(T::Output,)> for TryFromOutputFn<T>
where
    T: Try,
{
    type Output = T;

    fn call_mut(&mut self, args: (T::Output,)) -> Self::Output {
        T::from_output(args.0)
    }
}
