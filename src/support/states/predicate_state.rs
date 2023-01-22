use core::pin::Pin;

pin_project_lite::pin_project! {
    #[derive(Clone)]
    #[project = PredicateStateProject]
    #[project_replace = PredicateStateReplace]
    pub enum PredicateState<T, Fut> {
        Empty,
        Polling {
            item: T,
            #[pin]
            fut: Fut,
        }
    }
}

impl<T, Fut> PredicateState<T, Fut> {
    pub fn get_future(self: Pin<&mut Self>) -> Option<Pin<&mut Fut>> {
        match self.project() {
            PredicateStateProject::Empty => None,
            PredicateStateProject::Polling { fut, .. } => Some(fut),
        }
    }

    pub fn take_item(self: Pin<&mut Self>) -> Option<T> {
        match self.project_replace(Self::Empty) {
            PredicateStateReplace::Empty => None,
            PredicateStateReplace::Polling { item, .. } => Some(item),
        }
    }
}
