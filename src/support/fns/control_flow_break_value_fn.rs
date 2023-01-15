use crate::support::fn_mut_1::FnMut1;
use core::marker::PhantomData;
use core::ops::ControlFlow;

#[derive(Clone, Default)]
pub struct ControlFlowBreakValueFn {
    _phantom: PhantomData<()>,
}

impl<T> FnMut1<ControlFlow<T>> for ControlFlowBreakValueFn {
    type Output = Option<T>;

    fn call_mut(&mut self, arg: ControlFlow<T>) -> Self::Output {
        match arg {
            ControlFlow::Continue(()) => None,
            ControlFlow::Break(arg) => Some(arg),
        }
    }
}
