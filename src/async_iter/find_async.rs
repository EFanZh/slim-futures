use crate::support::states::PredicateState;
use crate::support::{AsyncIterator, FusedAsyncIterator, PredicateFn};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{self, Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct FindAsync<I, P>
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

impl<I, P> FindAsync<I, P>
where
    I: AsyncIterator,
    P: PredicateFn<I::Item>,
    <P as PredicateFn<I::Item>>::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            iter,
            state: PredicateState::Empty,
            predicate,
        }
    }
}

impl<I, P> Clone for FindAsync<I, P>
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
            state: self.state.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

impl<I, P> Future for FindAsync<I, P>
where
    I: AsyncIterator,
    P: PredicateFn<I::Item> + ?Sized,
    <P as PredicateFn<I::Item>>::Output: IntoFuture<Output = bool>,
{
    type Output = Option<I::Item>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let mut iter = this.iter;
        let predicate = this.predicate;
        let mut state_slot = this.state;

        Poll::Ready(loop {
            if let Some((result, item)) = task::ready!(state_slot.as_mut().try_poll(cx)) {
                if result {
                    break Some(item);
                }
            } else if let Some(item) = task::ready!(iter.as_mut().poll_next(cx)) {
                let fut = predicate.call_mut((&item,)).into_future();

                state_slot.set(PredicateState::Polling { item, fut });
            } else {
                break None;
            }
        })
    }
}

impl<I, P> FusedFuture for FindAsync<I, P>
where
    I: FusedAsyncIterator,
    P: PredicateFn<I::Item> + ?Sized,
    <P as PredicateFn<I::Item>>::Output: IntoFuture<Output = bool>,
    <<P as PredicateFn<I::Item>>::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        if let PredicateState::Polling { fut, .. } = &self.state {
            fut.is_terminated()
        } else {
            self.iter.is_terminated()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::{future, stream};

    #[tokio::test]
    async fn test_find_async() {
        let future = stream::iter([2, 3, 5]).slim_find_async(|&x| future::ready(x > 2));

        assert_eq!(future.await, Some(3));
    }

    #[tokio::test]
    async fn test_find_async_fail() {
        let future = stream::iter([2, 3, 5]).slim_find_async(|&x| future::ready(x < 1));

        assert!(future.await.is_none());
    }

    #[tokio::test]
    async fn test_find_async_clone() {
        let future = stream::iter([2, 3, 5]).slim_find_async(|&x| future::ready(x > 2));
        let future_2 = future.clone();

        assert_eq!(future.await, Some(3));
        assert_eq!(future_2.await, Some(3));
    }
}
