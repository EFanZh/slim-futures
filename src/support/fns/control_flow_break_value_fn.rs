use core::marker::PhantomData;
use core::ops::ControlFlow;
use fn_traits::FnMut;

#[derive(Clone, Default)]
pub struct ControlFlowBreakValueFn {
    _phantom: PhantomData<()>,
}

impl<T> FnMut<(ControlFlow<T>,)> for ControlFlowBreakValueFn {
    type Output = Option<T>;

    fn call_mut(&mut self, args: (ControlFlow<T>,)) -> Self::Output {
        match args.0 {
            ControlFlow::Continue(()) => None,
            ControlFlow::Break(value) => Some(value),
        }
    }
}
