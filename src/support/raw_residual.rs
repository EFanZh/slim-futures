use crate::support::try_::{FromResidual, Try};
use core::ops::ControlFlow;

pub struct RawResidual<B, C> {
    inner: ControlFlow<B, C>,
}

impl<B, C> FromResidual for RawResidual<B, C> {
    fn from_residual(residual: B) -> Self {
        Self {
            inner: ControlFlow::Break(residual),
        }
    }
}

impl<B, C> Try for RawResidual<B, C> {
    type Output = C;
    type Residual = B;

    fn from_output(output: Self::Output) -> Self {
        Self {
            inner: ControlFlow::Continue(output),
        }
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        self.inner
    }
}
