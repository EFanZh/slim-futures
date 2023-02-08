use crate::support::{FusedAsyncIterator, Never};
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::FusedFuture;
use three_states::{ThreeStates, ThreeStatesPinProject};

pin_project_lite::pin_project! {
    #[project = TwoPhasesProject]
    #[derive(Clone)]
    pub struct TwoPhases<A, B> {
        #[pin]
        inner: ThreeStates<A, (), B, (), Never, Never>,
    }
}

impl<A, B> TwoPhases<A, B> {
    pub fn new(state: A) -> Self {
        Self {
            inner: ThreeStates::A {
                pinned: state,
                unpinned: (),
            },
        }
    }

    pub fn poll_with<T>(
        self: Pin<&mut Self>,
        cx: &mut Context,
        f1: impl FnOnce(A::Output) -> ControlFlow<T, B>,
        f2: impl FnOnce(Pin<&mut B>, &mut Context) -> Poll<T>,
    ) -> Poll<T>
    where
        A: Future,
    {
        let second_state = match self.project().inner.pin_project() {
            ThreeStatesPinProject::A(mut project) => match f1(task::ready!(project.get_project().pinned.poll(cx))) {
                ControlFlow::Continue(second_state) => project.replace_state_b(second_state, ()).0,
                ControlFlow::Break(result) => return Poll::Ready(result),
            },
            ThreeStatesPinProject::B(project) => project,
            ThreeStatesPinProject::C(project) => match *project.into_project().unpinned {},
        };

        f2(second_state.into_project().pinned, cx)
    }

    pub fn get_second_phase(&self) -> Option<&B> {
        match &self.inner {
            ThreeStates::B { pinned, .. } => Some(pinned),
            _ => None,
        }
    }

    pub fn is_future_terminated(&self) -> bool
    where
        A: FusedFuture,
        B: FusedFuture,
    {
        match &self.inner {
            ThreeStates::A { pinned, .. } => pinned.is_terminated(),
            ThreeStates::B { pinned, .. } => pinned.is_terminated(),
            ThreeStates::C { unpinned, .. } => match *unpinned {},
        }
    }

    pub fn is_async_iter_terminated(&self) -> bool
    where
        A: FusedFuture,
        B: FusedAsyncIterator,
    {
        match &self.inner {
            ThreeStates::A { pinned, .. } => pinned.is_terminated(),
            ThreeStates::B { pinned, .. } => pinned.is_terminated(),
            ThreeStates::C { unpinned, .. } => match *unpinned {},
        }
    }
}
