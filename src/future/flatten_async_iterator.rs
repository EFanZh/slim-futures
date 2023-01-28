use crate::support::states::TwoPhases;
use crate::support::{AsyncIterator, FusedAsyncIterator, IntoAsyncIterator};
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct FlattenAsyncIterator<Fut>
    where
        Fut: Future,
        Fut::Output: IntoAsyncIterator,
    {
        #[pin]
        inner: TwoPhases<Fut, <Fut::Output as IntoAsyncIterator>::IntoAsyncIter>,
    }
}

impl<Fut> FlattenAsyncIterator<Fut>
where
    Fut: Future,
    Fut::Output: IntoAsyncIterator,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TwoPhases::First { state: fut },
        }
    }
}

impl<Fut> Clone for FlattenAsyncIterator<Fut>
where
    Fut: Future + Clone,
    Fut::Output: IntoAsyncIterator,
    <Fut::Output as IntoAsyncIterator>::IntoAsyncIter: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut> AsyncIterator for FlattenAsyncIterator<Fut>
where
    Fut: Future,
    Fut::Output: IntoAsyncIterator,
{
    type Item = <Fut::Output as IntoAsyncIterator>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        fn dispatch<I>(iter: I) -> ControlFlow<Option<I::Item>, I::IntoAsyncIter>
        where
            I: IntoAsyncIterator,
        {
            ControlFlow::Continue(iter.into_async_iter())
        }

        self.project().inner.poll_with(
            cx,
            dispatch,
            <Fut::Output as IntoAsyncIterator>::IntoAsyncIter::poll_next,
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            TwoPhases::First { .. } => (0, None),
            TwoPhases::Second { state } => state.size_hint(),
        }
    }
}

impl<Fut> FusedAsyncIterator for FlattenAsyncIterator<Fut>
where
    Fut: FusedFuture,
    Fut::Output: FusedAsyncIterator,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_async_iter_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use crate::support::FusedAsyncIterator;
    use futures_util::{future, stream, FutureExt as _, StreamExt};
    use std::mem;
    use std::num::NonZeroU32;

    fn make_flatten_async_iterator() -> impl FusedAsyncIterator<Item = u32> {
        future::ready(stream::once(future::ready(2))).slim_flatten_async_iterator()
    }

    #[tokio::test]
    async fn test_flatten_async_iterator() {
        let mut iter = make_flatten_async_iterator();

        assert_eq!(iter.next().await, Some(2));
        assert_eq!(iter.next().await, None);
    }

    #[tokio::test]
    async fn test_flatten_fused_async_iter() {
        let mut iter = make_flatten_async_iterator();

        assert!(!iter.is_terminated());
        assert_eq!(iter.next().await, Some(2));
        assert!(iter.is_terminated());
        assert_eq!(iter.next().await, None);
        assert!(iter.is_terminated());
    }

    #[tokio::test]
    async fn test_flatten_async_iterator_is_slim() {
        let make_base_future =
            || crate::future::ready_by_copy(NonZeroU32::new(2).unwrap()).slim_map(|_| crate::future::ready_by_copy(()));

        let base_future = make_base_future();
        let future_1 = make_base_future().slim_flatten();
        let future_2 = make_base_future().flatten();

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert!(matches!(base_future.await.await, ()));
        assert!(matches!(future_1.await, ()));
        assert!(matches!(future_2.await, ()));
    }
}
