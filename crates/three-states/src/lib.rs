use core::hint;
use core::pin::Pin;

pin_project_lite::pin_project! {
    #[derive(Clone, Copy)]
    #[project = InnerThreeStatesProject]
    #[project_replace = ThreeStatesProjectReplace]
    pub enum ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
        A { #[pin] pinned: APin, unpinned: AUnpin },
        B { #[pin] pinned: BPin, unpinned: BUnpin },
        C { #[pin] pinned: CPin, unpinned: CUnpin },
    }
}

impl<APin, AUnpin, BPin, BUnpin, CPin, CUnpin> ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    pub fn pin_project(self: Pin<&mut Self>) -> ThreeStatesProject<APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
        match *self {
            Self::A { .. } => ThreeStatesProject::A(StateAProject { inner: self }),
            Self::B { .. } => ThreeStatesProject::B(StateBProject { inner: self }),
            Self::C { .. } => ThreeStatesProject::C(StateCProject { inner: self }),
        }
    }
}

pub struct StateProject<'a, T, U> {
    pub pinned: Pin<&'a mut T>,
    pub unpinned: &'a mut U,
}

pub struct StateAProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    inner: Pin<&'a mut ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin>>,
}

impl<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> StateAProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    pub fn get_project(&mut self) -> StateProject<APin, AUnpin> {
        match self.inner.as_mut().project() {
            InnerThreeStatesProject::A { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn into_project(self) -> StateProject<'a, APin, AUnpin> {
        match self.inner.project() {
            InnerThreeStatesProject::A { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_a(&mut self, pinned: APin, unpinned: AUnpin) -> AUnpin {
        match self.inner.as_mut().project_replace(ThreeStates::A { pinned, unpinned }) {
            ThreeStatesProjectReplace::A { unpinned, .. } => unpinned,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_b(
        mut self,
        pinned: BPin,
        unpinned: BUnpin,
    ) -> (AUnpin, StateBProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::B { pinned, unpinned }) {
            ThreeStatesProjectReplace::A { unpinned, .. } => (unpinned, StateBProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_c(
        mut self,
        pinned: CPin,
        unpinned: CUnpin,
    ) -> (AUnpin, StateCProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::C { pinned, unpinned }) {
            ThreeStatesProjectReplace::A { unpinned, .. } => (unpinned, StateCProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

pub struct StateBProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    inner: Pin<&'a mut ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin>>,
}

impl<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> StateBProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    pub fn get_project(&mut self) -> StateProject<BPin, BUnpin> {
        match self.inner.as_mut().project() {
            InnerThreeStatesProject::B { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn into_project(self) -> StateProject<'a, BPin, BUnpin> {
        match self.inner.project() {
            InnerThreeStatesProject::B { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_a(
        mut self,
        pinned: APin,
        unpinned: AUnpin,
    ) -> (BUnpin, StateAProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::A { pinned, unpinned }) {
            ThreeStatesProjectReplace::B { unpinned, .. } => (unpinned, StateAProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_b(&mut self, pinned: BPin, unpinned: BUnpin) -> BUnpin {
        match self.inner.as_mut().project_replace(ThreeStates::B { pinned, unpinned }) {
            ThreeStatesProjectReplace::B { unpinned, .. } => unpinned,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_c(
        mut self,
        pinned: CPin,
        unpinned: CUnpin,
    ) -> (BUnpin, StateCProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::C { pinned, unpinned }) {
            ThreeStatesProjectReplace::B { unpinned, .. } => (unpinned, StateCProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

pub struct StateCProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    inner: Pin<&'a mut ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin>>,
}

impl<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> StateCProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    pub fn get_project(&mut self) -> StateProject<CPin, CUnpin> {
        match self.inner.as_mut().project() {
            InnerThreeStatesProject::C { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn into_project(self) -> StateProject<'a, CPin, CUnpin> {
        match self.inner.project() {
            InnerThreeStatesProject::C { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_a(
        mut self,
        pinned: APin,
        unpinned: AUnpin,
    ) -> (CUnpin, StateAProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::A { pinned, unpinned }) {
            ThreeStatesProjectReplace::C { unpinned, .. } => (unpinned, StateAProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_b(
        mut self,
        pinned: BPin,
        unpinned: BUnpin,
    ) -> (CUnpin, StateBProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::B { pinned, unpinned }) {
            ThreeStatesProjectReplace::C { unpinned, .. } => (unpinned, StateBProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_c(&mut self, pinned: CPin, unpinned: CUnpin) -> CUnpin {
        match self.inner.as_mut().project_replace(ThreeStates::C { pinned, unpinned }) {
            ThreeStatesProjectReplace::C { unpinned, .. } => unpinned,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

pub enum ThreeStatesProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    A(StateAProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>),
    B(StateBProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>),
    C(StateCProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>),
}
