use crate::support::{FromResidual, Try};
use core::ops::ControlFlow;

pub enum RawResidual<B, C> {
    Continue(C),
    Break(B),
}

impl<B, C> FromResidual for RawResidual<B, C> {
    fn from_residual(residual: B) -> Self {
        Self::Break(residual)
    }
}

impl<B, C> Try for RawResidual<B, C> {
    type Output = C;
    type Residual = B;

    fn from_output(output: Self::Output) -> Self {
        Self::Continue(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Continue(output) => ControlFlow::Continue(output),
            Self::Break(residual) => ControlFlow::Break(residual),
        }
    }
}
