use crate::support::try_::{FromResidual, Residual, Try};
use core::ops::ControlFlow;
use fn_traits::FnMut;

#[derive(Clone)]
pub struct MapOkFn<F>
where
    F: ?Sized,
{
    f: F,
}

impl<F> MapOkFn<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<T, F> FnMut<(T,)> for MapOkFn<F>
where
    T: Try,
    T::Residual: Residual<F::Output>,
    F: FnMut<(T::Output,)> + ?Sized,
{
    type Output = <T::Residual as Residual<F::Output>>::TryType;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        match args.0.branch() {
            ControlFlow::Continue(output) => Self::Output::from_output(self.f.call_mut((output,))),
            ControlFlow::Break(residual) => Self::Output::from_residual(residual),
        }
    }
}
