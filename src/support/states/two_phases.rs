use crate::support::FusedAsyncIterator;
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    #[project = TwoPhasesProject]
    pub enum TwoPhases<A, B> {
        First {
            #[pin]
            state: A,
        },
        Second {
            #[pin]
            state: B,
        },
    }
}

impl<A, B> TwoPhases<A, B> {
    pub fn poll_with<T>(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        f1: impl FnOnce(A::Output) -> ControlFlow<T, B>,
        f2: impl FnOnce(Pin<&mut B>, &mut Context) -> Poll<T>,
    ) -> Poll<T>
    where
        A: Future,
    {
        if let TwoPhasesProject::First { state } = self.as_mut().project() {
            let second_state = match f1(task::ready!(state.poll(cx))) {
                ControlFlow::Continue(second_state) => second_state,
                ControlFlow::Break(result) => return Poll::Ready(result),
            };

            self.set(Self::Second { state: second_state });
        }

        if let TwoPhasesProject::Second { state } = self.project() {
            f2(state, cx)
        } else {
            unreachable!()
        }
    }

    pub fn is_future_terminated(&self) -> bool
    where
        A: FusedFuture,
        B: FusedFuture,
    {
        match self {
            Self::First { state } => state.is_terminated(),
            Self::Second { state } => state.is_terminated(),
        }
    }

    pub fn is_async_iter_terminated(&self) -> bool
    where
        A: FusedFuture,
        B: FusedAsyncIterator,
    {
        match self {
            Self::First { state } => state.is_terminated(),
            Self::Second { state } => state.is_terminated(),
        }
    }
}

impl<A, B> Clone for TwoPhases<A, B>
where
    A: Clone,
    B: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::First { state } => Self::First { state: state.clone() },
            Self::Second { state } => Self::Second { state: state.clone() },
        }
    }
}
