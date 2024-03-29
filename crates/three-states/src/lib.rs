use core::pin::Pin;
use core::{hint, mem};

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

    pub fn project_mut(&mut self) -> ThreeStatesProject<APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
        match *self {
            Self::A { .. } => ThreeStatesProject::A(StateAProject { inner: self }),
            Self::B { .. } => ThreeStatesProject::B(StateBProject { inner: self }),
            Self::C { .. } => ThreeStatesProject::C(StateCProject { inner: self }),
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

    pub fn replace_state_a(&mut self, pinned: APin, unpinned: AUnpin) -> AUnpin {
        match self.inner.as_mut().project_replace(ThreeStates::A { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::A { unpinned, .. } => unpinned,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_b(
        mut self,
        pinned: BPin,
        unpinned: BUnpin,
    ) -> (StateBPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, AUnpin) {
        match self.inner.as_mut().project_replace(ThreeStates::B { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::A { unpinned, .. } => (StateBPinProject { inner: self.inner }, unpinned),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_c(
        mut self,
        pinned: CPin,
        unpinned: CUnpin,
    ) -> (StateCPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, AUnpin) {
        match self.inner.as_mut().project_replace(ThreeStates::C { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::A { unpinned, .. } => (StateCPinProject { inner: self.inner }, unpinned),
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

    pub fn replace_state_a(
        mut self,
        pinned: APin,
        unpinned: AUnpin,
    ) -> (StateAPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, BUnpin) {
        match self.inner.as_mut().project_replace(ThreeStates::A { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::B { unpinned, .. } => (StateAPinProject { inner: self.inner }, unpinned),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_b(&mut self, pinned: BPin, unpinned: BUnpin) -> BUnpin {
        match self.inner.as_mut().project_replace(ThreeStates::B { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::B { unpinned, .. } => unpinned,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_c(
        mut self,
        pinned: CPin,
        unpinned: CUnpin,
    ) -> (StateCPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, BUnpin) {
        match self.inner.as_mut().project_replace(ThreeStates::C { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::B { unpinned, .. } => (StateCPinProject { inner: self.inner }, unpinned),
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

    pub fn replace_state_a(
        mut self,
        pinned: APin,
        unpinned: AUnpin,
    ) -> (StateAPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, CUnpin) {
        match self.inner.as_mut().project_replace(ThreeStates::A { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::C { unpinned, .. } => (StateAPinProject { inner: self.inner }, unpinned),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_b(
        mut self,
        pinned: BPin,
        unpinned: BUnpin,
    ) -> (StateBPinProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, CUnpin) {
        match self.inner.as_mut().project_replace(ThreeStates::B { pinned, unpinned }) {
            ThreeStatesPinProjectReplace::C { unpinned, .. } => (StateBPinProject { inner: self.inner }, unpinned),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_c(&mut self, pinned: CPin, unpinned: CUnpin) -> CUnpin {
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

pub struct StateProject<'a, A, B> {
    pub pinned: &'a mut A,
    pub unpinned: &'a mut B,
}

pub struct StateAProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    inner: &'a mut ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin>,
}

impl<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> StateAProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    pub fn get_project(&mut self) -> StateProject<APin, AUnpin> {
        match self.inner {
            ThreeStates::A { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn into_project(self) -> StateProject<'a, APin, AUnpin> {
        match self.inner {
            ThreeStates::A { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_a(&mut self, pinned: APin, unpinned: AUnpin) -> AUnpin {
        match mem::replace(self.inner, ThreeStates::A { pinned, unpinned }) {
            ThreeStates::A { unpinned, .. } => unpinned,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_b(
        self,
        pinned: BPin,
        unpinned: BUnpin,
    ) -> (StateBProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, AUnpin) {
        match mem::replace(self.inner, ThreeStates::B { pinned, unpinned }) {
            ThreeStates::A { unpinned, .. } => (StateBProject { inner: self.inner }, unpinned),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_c(
        self,
        pinned: CPin,
        unpinned: CUnpin,
    ) -> (StateCProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, AUnpin) {
        match mem::replace(self.inner, ThreeStates::C { pinned, unpinned }) {
            ThreeStates::A { unpinned, .. } => (StateCProject { inner: self.inner }, unpinned),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

pub struct StateBProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    inner: &'a mut ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin>,
}

impl<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> StateBProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    pub fn get_project(&mut self) -> StateProject<BPin, BUnpin> {
        match self.inner {
            ThreeStates::B { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn into_project(self) -> StateProject<'a, BPin, BUnpin> {
        match self.inner {
            ThreeStates::B { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_a(
        self,
        pinned: APin,
        unpinned: AUnpin,
    ) -> (StateAProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, BUnpin) {
        match mem::replace(self.inner, ThreeStates::A { pinned, unpinned }) {
            ThreeStates::B { unpinned, .. } => (StateAProject { inner: self.inner }, unpinned),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_b(&mut self, pinned: BPin, unpinned: BUnpin) -> BUnpin {
        match mem::replace(self.inner, ThreeStates::B { pinned, unpinned }) {
            ThreeStates::B { unpinned, .. } => unpinned,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_c(
        self,
        pinned: CPin,
        unpinned: CUnpin,
    ) -> (StateCProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, BUnpin) {
        match mem::replace(self.inner, ThreeStates::C { pinned, unpinned }) {
            ThreeStates::B { unpinned, .. } => (StateCProject { inner: self.inner }, unpinned),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

pub struct StateCProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    inner: &'a mut ThreeStates<APin, AUnpin, BPin, BUnpin, CPin, CUnpin>,
}

impl<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> StateCProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    pub fn get_project(&mut self) -> StateProject<CPin, CUnpin> {
        match self.inner {
            ThreeStates::C { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn into_project(self) -> StateProject<'a, CPin, CUnpin> {
        match self.inner {
            ThreeStates::C { pinned, unpinned } => StateProject { pinned, unpinned },
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_a(
        self,
        pinned: APin,
        unpinned: AUnpin,
    ) -> (StateAProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, CUnpin) {
        match mem::replace(self.inner, ThreeStates::A { pinned, unpinned }) {
            ThreeStates::C { unpinned, .. } => (StateAProject { inner: self.inner }, unpinned),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_b(
        self,
        pinned: BPin,
        unpinned: BUnpin,
    ) -> (StateBProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>, CUnpin) {
        match mem::replace(self.inner, ThreeStates::B { pinned, unpinned }) {
            ThreeStates::C { unpinned, .. } => (StateBProject { inner: self.inner }, unpinned),
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub fn replace_state_c(&mut self, pinned: CPin, unpinned: CUnpin) -> CUnpin {
        match mem::replace(self.inner, ThreeStates::C { pinned, unpinned }) {
            ThreeStates::C { unpinned, .. } => unpinned,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

pub enum ThreeStatesProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin> {
    A(StateAProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>),
    B(StateBProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>),
    C(StateCProject<'a, APin, AUnpin, BPin, BUnpin, CPin, CUnpin>),
}
