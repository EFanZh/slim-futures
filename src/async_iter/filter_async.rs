use crate::support::{AsyncIterator, FnMut1, FusedAsyncIterator};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::{FusedFuture, Future};

trait PredicateFn<T>: for<'a> FnMut1<&'a T> {}

impl<T, F> PredicateFn<T> for F where F: for<'a> FnMut1<&'a T> {}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    #[project = PredicateStateProject]
    #[project_replace = PredicateStateReplace]
    enum PredicateState<T, Fut> {
        Empty,
        Polling {
            item: T,
            #[pin]
            fut: Fut,
        }
    }
}

pin_project_lite::pin_project! {
    pub struct FilterAsync<I, P, Fut>
    where
        I: AsyncIterator,
    {
        #[pin]
        iter: I,
        predicate: P,
        #[pin]
        state: PredicateState<I::Item, Fut>,
    }
}

impl<I, P, Fut> FilterAsync<I, P, Fut>
where
    I: AsyncIterator,
{
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            iter,
            predicate,
            state: PredicateState::Empty,
        }
    }
}

impl<I, P, Fut> Clone for FilterAsync<I, P, Fut>
where
    I: AsyncIterator + Clone,
    P: Clone,
    I::Item: Clone,
    Fut: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            predicate: self.predicate.clone(),
            state: self.state.clone(),
        }
    }
}

impl<I, P, Fut> AsyncIterator for FilterAsync<I, P, Fut>
where
    I: AsyncIterator,
    P: for<'a> FnMut1<&'a I::Item, Output = Fut>,
    Fut: Future<Output = bool>,
{
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let predicate = this.predicate;
        let mut state_slot = this.state;

        Poll::Ready(loop {
            match state_slot.as_mut().project() {
                PredicateStateProject::Empty => {}
                PredicateStateProject::Polling { fut, .. } => {
                    let filter_result = task::ready!(fut.poll(cx));

                    let item = match state_slot.as_mut().project_replace(PredicateState::Empty) {
                        PredicateStateReplace::Empty => unreachable!(),
                        PredicateStateReplace::Polling { item, .. } => item,
                    };

                    if filter_result {
                        break Some(item);
                    }
                }
            }

            match task::ready!(iter.as_mut().poll_next(cx)) {
                None => break None,
                Some(item) => {
                    let fut = predicate.call_mut(&item);

                    state_slot.set(PredicateState::Polling { item, fut });
                }
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let high = self.iter.size_hint().1;

        let high = match &self.state {
            PredicateState::Empty => high,
            PredicateState::Polling { .. } => high.and_then(|high| high.checked_add(1)),
        };

        (0, high)
    }
}

impl<I, P, Fut> FusedAsyncIterator for FilterAsync<I, P, Fut>
where
    I: FusedAsyncIterator,
    P: for<'a> FnMut1<&'a I::Item, Output = Fut>,
    Fut: FusedFuture<Output = bool>,
{
    fn is_terminated(&self) -> bool {
        match &self.state {
            PredicateState::Empty => self.iter.is_terminated(),
            PredicateState::Polling { fut, .. } => fut.is_terminated(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::{future, stream, StreamExt};
    use std::vec::Vec;

    #[tokio::test]
    async fn test_filter_async() {
        let iter = stream::iter(0..10).slim_filter_async(|&x| future::ready(x % 2 == 0));

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_filter_async_clone() {
        let iter = stream::iter(0..10).slim_filter_async(|&x| future::ready(x % 2 == 0));
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
    }
}
