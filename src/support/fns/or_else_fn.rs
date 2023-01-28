use crate::support::try_::Try;
use fn_traits::FnMut;

#[derive(Clone)]
pub struct OrElseFn<F>
where
    F: ?Sized,
{
    f: F,
}

impl<F> OrElseFn<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<T, E, F> FnMut<(Result<T, E>,)> for OrElseFn<F>
where
    F: FnMut<(E,)> + ?Sized,
    F::Output: Try<Output = T>,
{
    type Output = F::Output;

    fn call_mut(&mut self, args: (Result<T, E>,)) -> Self::Output {
        args.0
            .map_or_else(|error| self.f.call_mut((error,)), Self::Output::from_output)
    }
}
