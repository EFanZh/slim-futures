use crate::support::fn_mut_1::FnMut1;
use core::marker::PhantomData;
use core::ops::ControlFlow;

#[derive(Clone, Default)]
pub struct BoolToControlFlowAllFn {
    _phantom: PhantomData<()>,
}

impl FnMut1<bool> for BoolToControlFlowAllFn {
    type Output = ControlFlow<()>;

    fn call_mut(&mut self, arg: bool) -> Self::Output {
        if arg {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}
