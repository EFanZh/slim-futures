use core::future::Future;
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
    pub fn set_polling(mut self: Pin<&mut Self>, item: T, fut: Fut) -> Pin<&mut Fut>
    where
        Fut: Future,
    {
        self.set(Self::Polling { item, fut });

        match self.project() {
            PredicateStateProject::Empty => unreachable!(),
            PredicateStateProject::Polling { fut, .. } => fut,
        }
    }
}
