use crate::support::Never;
use core::pin::Pin;
use three_states::{StateAPinProject, StateBPinProject, ThreeStates, ThreeStatesPinProject};

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct FoldState<T, Fut> {
        #[pin]
        inner: ThreeStates<(), T, Fut, (), Never, Never>,
    }
}

impl<T, Fut> FoldState<T, Fut> {
    pub fn new(acc: T) -> Self {
        Self {
            inner: ThreeStates::A {
                pinned: (),
                unpinned: acc,
            },
        }
    }

    pub fn get_future(&self) -> Option<&Fut> {
        match &self.inner {
            ThreeStates::B { pinned, .. } => Some(pinned),
            _ => None,
        }
    }

    pub fn pin_project(self: Pin<&mut Self>) -> FoldStateProject<T, Fut> {
        match self.project().inner.pin_project() {
            ThreeStatesPinProject::A(project) => FoldStateProject::Accumulate(FoldAccumulateState { inner: project }),
            ThreeStatesPinProject::B(project) => FoldStateProject::Future(FoldFutureState { inner: project }),
            ThreeStatesPinProject::C(project) => match *project.into_project().unpinned {},
        }
    }
}

pub struct FoldAccumulateState<'a, T, Fut> {
    inner: StateAPinProject<'a, (), T, Fut, (), Never, Never>,
}

impl<'a, T, Fut> FoldAccumulateState<'a, T, Fut> {
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_project().unpinned
    }

    pub fn set_future(self, fut: Fut) -> FoldFutureState<'a, T, Fut> {
        FoldFutureState {
            inner: self.inner.set_state_b(fut, ()).1,
        }
    }
}

pub struct FoldFutureState<'a, T, Fut> {
    inner: StateBPinProject<'a, (), T, Fut, (), Never, Never>,
}

impl<'a, T, Fut> FoldFutureState<'a, T, Fut> {
    pub fn get_pinned(&mut self) -> Pin<&mut Fut> {
        self.inner.get_project().pinned
    }

    pub fn set_accumulate(self, acc: T) -> FoldAccumulateState<'a, T, Fut> {
        FoldAccumulateState {
            inner: self.inner.set_state_a((), acc).1,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum FoldStateProject<'a, T, Fut> {
    Accumulate(FoldAccumulateState<'a, T, Fut>),
    Future(FoldFutureState<'a, T, Fut>),
}
