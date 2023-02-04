use crate::support::Never;
use core::pin::Pin;
use three_states::{StateAProject, StateBProject, ThreeStates, ThreeStatesProject};

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct PredicateState<T, Fut> {
        #[pin]
        inner: ThreeStates<(), (), Fut, T, Never, Never>,
    }
}

impl<T, Fut> PredicateState<T, Fut> {
    pub fn get_future(&self) -> Option<&Fut> {
        match &self.inner {
            ThreeStates::B { pinned, .. } => Some(pinned),
            _ => None,
        }
    }

    pub fn pin_project(self: Pin<&mut Self>) -> PredicateStateProject<T, Fut> {
        match self.project().inner.pin_project() {
            ThreeStatesProject::A(project) => PredicateStateProject::Empty(PredicateEmptyState { inner: project }),
            ThreeStatesProject::B(project) => PredicateStateProject::Future(PredicateFutureState { inner: project }),
            ThreeStatesProject::C(project) => match *project.into_project().unpinned {},
        }
    }
}

impl<T, Fut> Default for PredicateState<T, Fut> {
    fn default() -> Self {
        Self {
            inner: ThreeStates::A {
                pinned: (),
                unpinned: (),
            },
        }
    }
}

pub struct PredicateEmptyState<'a, T, Fut> {
    inner: StateAProject<'a, (), (), Fut, T, Never, Never>,
}

impl<'a, T, Fut> PredicateEmptyState<'a, T, Fut> {
    pub fn set_future(self, item: T, fut: Fut) -> PredicateFutureState<'a, T, Fut> {
        PredicateFutureState {
            inner: self.inner.set_state_b(fut, item).1,
        }
    }
}

pub struct PredicateFutureState<'a, T, Fut> {
    inner: StateBProject<'a, (), (), Fut, T, Never, Never>,
}

impl<'a, T, Fut> PredicateFutureState<'a, T, Fut> {
    pub fn get_pinned_future(&mut self) -> Pin<&mut Fut> {
        self.inner.get_project().pinned
    }

    pub fn set_empty(self) -> (T, PredicateEmptyState<'a, T, Fut>) {
        let (item, inner) = self.inner.set_state_a((), ());

        (item, PredicateEmptyState { inner })
    }
}

#[allow(clippy::module_name_repetitions)]
pub enum PredicateStateProject<'a, T, Fut> {
    Empty(PredicateEmptyState<'a, T, Fut>),
    Future(PredicateFutureState<'a, T, Fut>),
}
