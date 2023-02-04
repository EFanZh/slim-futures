use core::hint;
use core::pin::Pin;

pin_project_lite::pin_project! {
    #[derive(Clone, Copy)]
    #[project = InnerThreeStatesPinProject]
    #[project_replace = ThreeStatesPinProjectReplace]
    pub enum ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
        A { #[pin] pinned: APin, unpinned: AUnpin },
        B { #[pin] pinned: BPin, unpinned: BUnpin },
        C { #[pin] pinned: CPin, unpinned: CUnpin },
    }
}

impl<APin, AUnpin, BPin, BUnpin, CPin, CUnpin> ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    pub fn pin_project(self: Pin<&mut Self>) -> ThreeStatesPinProject<APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
        match *self {
            Self::A { .. } => ThreeStatesPinProject::A(StateAPinProject { inner: self }),
            Self::B { .. } => ThreeStatesPinProject::B(StateBPinProject { inner: self }),
            Self::C { .. } => ThreeStatesPinProject::C(StateCPinProject { inner: self }),
        }
    }
}

pub struct StatePinProject<'a, A, B> {
    pub pinned: Pin<&'a mut A>,
    pub unpinned: &'a mut B,
}

pub struct StateAPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    inner: Pin<&'a mut ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin>>,
}

impl<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> StateAPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    pub fn get_project(&mut self) -> StatePinProject<APin, AUnpin> {
        match self.inner.as_mut().project() {
            InnerThreeStatesPinProject::A { pinned, unpinned } => StatePinProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn into_project(self) -> StatePinProject<'a, APin, AUnpin> {
        match self.inner.project() {
            InnerThreeStatesPinProject::A { pinned, unpinned } => StatePinProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_a(&mut self, pinned: APin, unpinned: AUnpin) -> AUnpin {
        match self.inner.as_mut().project_replace(ThreeStates::A { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::A { unpinned, .. } => unpinned,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_b(
        mut self,
        pinned: BPin,
        unpinned: BUnpin,
    ) -> (AUnpin, StateBPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::B { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::A { unpinned, .. } => (unpinned, StateBPinProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_c(
        mut self,
        pinned: CPin,
        unpinned: CUnpin,
    ) -> (AUnpin, StateCPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::C { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::A { unpinned, .. } => (unpinned, StateCPinProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

pub struct StateBPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    inner: Pin<&'a mut ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin>>,
}

impl<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> StateBPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    pub fn get_project(&mut self) -> StatePinProject<BPin, BUnpin> {
        match self.inner.as_mut().project() {
            InnerThreeStatesPinProject::B { pinned, unpinned } => StatePinProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn into_project(self) -> StatePinProject<'a, BPin, BUnpin> {
        match self.inner.project() {
            InnerThreeStatesPinProject::B { pinned, unpinned } => StatePinProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_a(
        mut self,
        pinned: APin,
        unpinned: AUnpin,
    ) -> (BUnpin, StateAPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::A { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::B { unpinned, .. } => (unpinned, StateAPinProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_b(&mut self, pinned: BPin, unpinned: BUnpin) -> BUnpin {
        match self.inner.as_mut().project_replace(ThreeStates::B { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::B { unpinned, .. } => unpinned,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_c(
        mut self,
        pinned: CPin,
        unpinned: CUnpin,
    ) -> (BUnpin, StateCPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::C { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::B { unpinned, .. } => (unpinned, StateCPinProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

pub struct StateCPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    inner: Pin<&'a mut ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin>>,
}

impl<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> StateCPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    pub fn get_project(&mut self) -> StatePinProject<CPin, CUnpin> {
        match self.inner.as_mut().project() {
            InnerThreeStatesPinProject::C { pinned, unpinned } => StatePinProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn into_project(self) -> StatePinProject<'a, CPin, CUnpin> {
        match self.inner.project() {
            InnerThreeStatesPinProject::C { pinned, unpinned } => StatePinProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_a(
        mut self,
        pinned: APin,
        unpinned: AUnpin,
    ) -> (CUnpin, StateAPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::A { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::C { unpinned, .. } => (unpinned, StateAPinProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_b(
        mut self,
        pinned: BPin,
        unpinned: BUnpin,
    ) -> (CUnpin, StateBPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>) {
        match self.inner.as_mut().project_replace(ThreeStates::B { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::C { unpinned, .. } => (unpinned, StateBPinProject { inner: self.inner }),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn set_state_c(&mut self, pinned: CPin, unpinned: CUnpin) -> CUnpin {
        match self.inner.as_mut().project_replace(ThreeStates::C { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::C { unpinned, .. } => unpinned,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

pub enum ThreeStatesPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    A(StateAPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>),
    B(StateBPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>),
    C(StateCPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>),
}
