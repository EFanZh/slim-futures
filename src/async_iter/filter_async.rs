use crate::support::{AsyncIterator, FnMut1, FusedAsyncIterator};
use core::future::IntoFuture;
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::{FusedFuture, Future};

pub trait PredicateFn<T>: for<'a> FnMut1<&'a T, Output = <Self as PredicateFn<T>>::Output> {
    type Output;
}

impl<T, F, R> PredicateFn<T> for F
where
    F: for<'a> FnMut1<&'a T, Output = R>,
{
    type Output = R;
}

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
    pub struct FilterAsync<I, P>
    where
        I: AsyncIterator,
        P: PredicateFn<I::Item>,
        <P as PredicateFn<I::Item>>::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        predicate: P,
        #[pin]
        state: PredicateState<I::Item, <<P as PredicateFn<I::Item>>::Output as IntoFuture>::IntoFuture>,
    }
}

impl<I, P> FilterAsync<I, P>
where
    I: AsyncIterator,
    P: PredicateFn<I::Item>,
    <P as PredicateFn<I::Item>>::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            iter,
            predicate,
            state: PredicateState::Empty,
        }
    }
}

impl<I, P> Clone for FilterAsync<I, P>
where
    I: AsyncIterator + Clone,
    I::Item: Clone,
    P: PredicateFn<I::Item> + Clone,
    <P as PredicateFn<I::Item>>::Output: IntoFuture,
    <<P as PredicateFn<I::Item>>::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            predicate: self.predicate.clone(),
            state: self.state.clone(),
        }
    }
}

impl<I, P> AsyncIterator for FilterAsync<I, P>
where
    I: AsyncIterator,
    P: PredicateFn<I::Item>,
    <P as PredicateFn<I::Item>>::Output: IntoFuture<Output = bool>,
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
                    let fut = predicate.call_mut(&item).into_future();

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

impl<I, P> FusedAsyncIterator for FilterAsync<I, P>
where
    I: FusedAsyncIterator,
    P: PredicateFn<I::Item>,
    <P as PredicateFn<I::Item>>::Output: IntoFuture<Output = bool>,
    <<P as PredicateFn<I::Item>>::Output as IntoFuture>::IntoFuture: FusedFuture,
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
