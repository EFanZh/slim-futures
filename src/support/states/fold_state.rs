use crate::support::Never;
use core::pin::Pin;
use three_states::{StateAProject, StateBProject, ThreeStates, ThreeStatesProject};

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
            ThreeStatesProject::A(project) => FoldStateProject::Accumulate(AccumulateState { inner: project }),
            ThreeStatesProject::B(project) => FoldStateProject::Future(FutureState { inner: project }),
            ThreeStatesProject::C(project) => match *project.into_project().unpinned {},
        }
    }
}

pub struct AccumulateState<'a, T, Fut> {
    inner: StateAProject<'a, (), T, Fut, (), Never, Never>,
}

impl<'a, T, Fut> AccumulateState<'a, T, Fut> {
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_project().unpinned
    }

    pub fn set_future(self, fut: Fut) -> FutureState<'a, T, Fut> {
        FutureState {
            inner: self.inner.set_state_b(fut, ()).1,
        }
    }
}

pub struct FutureState<'a, T, Fut> {
    inner: StateBProject<'a, (), T, Fut, (), Never, Never>,
}

impl<'a, T, Fut> FutureState<'a, T, Fut> {
    pub fn get_pinned(&mut self) -> Pin<&mut Fut> {
        self.inner.get_project().pinned
    }

    pub fn set_accumulate(self, acc: T) -> AccumulateState<'a, T, Fut> {
        AccumulateState {
            inner: self.inner.set_state_a((), acc).1,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum FoldStateProject<'a, T, Fut> {
    Accumulate(AccumulateState<'a, T, Fut>),
    Future(FutureState<'a, T, Fut>),
}
