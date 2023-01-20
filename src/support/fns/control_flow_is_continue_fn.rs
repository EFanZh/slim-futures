use core::marker::PhantomData;
use core::ops::ControlFlow;
use fn_traits::FnMut;

#[derive(Clone, Default)]
pub struct ControlFlowIsContinueFn {
    _phantom: PhantomData<()>,
}

impl FnMut<(ControlFlow<()>,)> for ControlFlowIsContinueFn {
    type Output = bool;

    fn call_mut(&mut self, args: (ControlFlow<()>,)) -> Self::Output {
        args.0.is_continue()
    }
}
