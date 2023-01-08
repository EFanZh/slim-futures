use crate::support::fn_mut_1::FnMut1;
use core::marker::PhantomData;
use core::ops::ControlFlow;

#[derive(Clone, Default)]
pub struct ControlFlowToBoolAllFn {
    _phantom: PhantomData<()>,
}

impl FnMut1<ControlFlow<()>> for ControlFlowToBoolAllFn {
    type Output = bool;

    fn call_mut(&mut self, arg: ControlFlow<()>) -> Self::Output {
        arg.is_continue()
    }
}
