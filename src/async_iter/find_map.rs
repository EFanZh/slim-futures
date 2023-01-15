use crate::async_iter::try_fold::TryFold;
use crate::future::Map;
use crate::support::fns::ControlFlowBreakValueFn;
use crate::support::{AsyncIterator, FnMut1, FnMut2, FusedAsyncIterator};
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

#[derive(Clone)]
struct FindMapFn<F> {
    inner: F,
}

impl<T, F, B> FnMut2<(), T> for FindMapFn<F>
where
    F: FnMut1<T, Output = Option<B>>,
{
    type Output = ControlFlow<B>;

    fn call_mut(&mut self, (): (), arg_2: T) -> Self::Output {
        match self.inner.call_mut(arg_2) {
            None => ControlFlow::Continue(()),
            Some(item) => ControlFlow::Break(item),
        }
    }
}

pin_project_lite::pin_project! {
    pub struct FindMap<I, F> {
        #[pin]
        inner: Map<TryFold<I, (), FindMapFn<F>>, ControlFlowBreakValueFn>
    }
}

impl<I, F> FindMap<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Map::new(
                TryFold::new(iter, (), FindMapFn { inner: f }),
                ControlFlowBreakValueFn::default(),
            ),
        }
    }
}

impl<I, F> Clone for FindMap<I, F>
where
    I: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F, B> Future for FindMap<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item, Output = Option<B>>,
{
    type Output = Option<B>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<I, F, B> FusedFuture for FindMap<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut1<I::Item, Output = Option<B>>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::stream;

    fn find_3_then_mul_10(x: u32) -> Option<u64> {
        (x == 3).then_some(u64::from(x) * 10)
    }

    fn find_7_then_mul_10(x: u32) -> Option<u64> {
        (x == 7).then_some(u64::from(x) * 10)
    }

    #[tokio::test]
    async fn test_find_map() {
        let future = stream::iter([2, 3, 5]).slim_find_map(find_3_then_mul_10);

        assert_eq!(future.await, Some(30));
    }

    #[tokio::test]
    async fn test_find_map_fail() {
        let future = stream::iter([2, 3, 5]).slim_find_map(find_7_then_mul_10);

        assert!(future.await.is_none());
    }

    #[tokio::test]
    async fn test_find_map_clone() {
        let future = stream::iter([2, 3, 5]).slim_find_map(find_3_then_mul_10);
        let future_2 = future.clone();

        assert_eq!(future.await, Some(30));
        assert_eq!(future_2.await, Some(30));
    }
}
