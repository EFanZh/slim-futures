use crate::support::try_::{FromResidual, Try};
use core::ops::ControlFlow;
use fn_traits::FnMut;

#[derive(Clone)]
pub struct AndThenFn<F>
where
    F: ?Sized,
{
    f: F,
}

impl<F> AndThenFn<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<T, F> FnMut<(T,)> for AndThenFn<F>
where
    T: Try,
    F: FnMut<(T::Output,)> + ?Sized,
    F::Output: FromResidual<T::Residual> + Try,
{
    type Output = F::Output;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        match args.0.branch() {
            ControlFlow::Continue(output) => self.f.call_mut((output,)),
            ControlFlow::Break(residual) => Self::Output::from_residual(residual),
        }
    }
}
