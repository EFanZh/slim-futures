use crate::support::Never;
use core::marker::PhantomData;
use core::ops::ControlFlow;
use fn_traits::FnMut;

#[derive(Clone, Default)]
pub struct UnwrapContinueValueFn {
    phantom: PhantomData<()>,
}

impl<T> FnMut<(ControlFlow<Never, T>,)> for UnwrapContinueValueFn {
    type Output = T;

    fn call_mut(&mut self, args: (ControlFlow<Never, T>,)) -> Self::Output {
        match args.0 {
            ControlFlow::Continue(value) => value,
            ControlFlow::Break(never) => match never {},
        }
    }
}
