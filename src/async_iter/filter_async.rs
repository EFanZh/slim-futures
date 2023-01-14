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
    enum PredicateState<T, F> {
        Empty,
        Polling {
            item: T,
            #[pin]
            future: F,
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
                PredicateStateProject::Polling { future, .. } => {
                    if task::ready!(future.poll(cx)) {
                        match state_slot.project_replace(PredicateState::Empty) {
                            PredicateStateReplace::Empty => unreachable!(),
                            PredicateStateReplace::Polling { item, .. } => break Some(item),
                        }
                    } else {
                        state_slot.set(PredicateState::Empty);
                    }
                }
            }

            match task::ready!(iter.as_mut().poll_next(cx)) {
                None => break None,
                Some(item) => {
                    let future = predicate.call_mut(&item);

                    state_slot.set(PredicateState::Polling { item, future });
                }
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
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
            PredicateState::Polling { future, .. } => future.is_terminated(),
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
        let iter = stream::iter(0..10).filter_async(|&x| future::ready(x % 2 == 0));

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_filter_async_clone() {
        let iter = stream::iter(0..10).filter_async(|&x| future::ready(x % 2 == 0));
        let iter_2 = iter.clone();

        assert_eq!(iter.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
        assert_eq!(iter_2.collect::<Vec<_>>().await, [0, 2, 4, 6, 8]);
    }
}
