use core::future::Future;
use core::pin::Pin;
use core::task::{self, Context, Poll};

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
    pub fn try_poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<(Fut::Output, T)>>
    where
        Fut: Future,
    {
        Poll::Ready(match self.as_mut().project() {
            PredicateStateProject::Empty => None,
            PredicateStateProject::Polling { fut, .. } => Some({
                let result = task::ready!(fut.poll(cx));

                match self.project_replace(Self::Empty) {
                    PredicateStateReplace::Empty => unreachable!(),
                    PredicateStateReplace::Polling { item, .. } => (result, item),
                }
            }),
        })
    }
}
