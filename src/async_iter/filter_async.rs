use crate::support::states::{PredicateState, PredicateStateProject};
use crate::support::{AsyncIterator, FusedAsyncIterator, PredicateFn};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct FilterAsync<I, P>
    where
        I: AsyncIterator,
        P: PredicateFn<I::Item>,
        P: ?Sized,
        <P as PredicateFn<I::Item>>::Output: IntoFuture,
    {
        #[pin]
        iter: I,
        #[pin]
        state: PredicateState<I::Item, <<P as PredicateFn<I::Item>>::Output as IntoFuture>::IntoFuture>,
        predicate: P,
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
            state: PredicateState::default(),
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
    P: PredicateFn<I::Item> + ?Sized,
    <P as PredicateFn<I::Item>>::Output: IntoFuture<Output = bool>,
{
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut iter = this.iter;
        let mut state = this.state.pin_project();
        let predicate = this.predicate;

        loop {
            let mut fut = match state {
                PredicateStateProject::Empty(empty_state) => match task::ready!(iter.as_mut().poll_next(cx)) {
                    None => break Poll::Ready(None),
                    Some(item) => {
                        let fut = predicate.call_mut((&item,)).into_future();

                        empty_state.set_future(item, fut)
                    }
                },
                PredicateStateProject::Future(fut_state) => fut_state,
            };

            let keep = task::ready!(fut.get_pin_mut().poll(cx));
            let (empty_state, item) = fut.set_empty();

            state = PredicateStateProject::Empty(empty_state);

            if keep {
                break Poll::Ready(Some(item));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut candidate = (0, self.iter.size_hint().1);

        if self.state.get_future().is_some() {
            candidate.1 = candidate.1.and_then(|high| high.checked_add(1));
        }

        candidate
    }
}

impl<I, P> FusedAsyncIterator for FilterAsync<I, P>
where
    I: FusedAsyncIterator,
    P: PredicateFn<I::Item> + ?Sized,
    <P as PredicateFn<I::Item>>::Output: IntoFuture<Output = bool>,
    <<P as PredicateFn<I::Item>>::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.state
            .get_future()
            .map_or_else(|| self.iter.is_terminated(), FusedFuture::is_terminated)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iterator_ext::AsyncIteratorExt;
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
