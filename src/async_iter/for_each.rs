use crate::async_iter::fold::Fold;
use crate::support::fns::ForEachFn;
use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::CopyFn;
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct ForEach<I, F> {
        #[pin]
        inner: Fold<I, (), CopyFn, ForEachFn<F>>,
    }
}

impl<I, F> ForEach<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Fold::new(iter, (), ForEachFn::new(f)),
        }
    }
}

impl<I, F> Clone for ForEach<I, F>
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

impl<I, F> Future for ForEach<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,), Output = ()>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<I, F> FusedFuture for ForEach<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut<(I::Item,), Output = ()>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::stream;
    use std::sync::Mutex;
    use std::vec::Vec;

    #[tokio::test]
    async fn test_for_each() {
        let mut result = Vec::new();
        let future = stream::iter([2, 3, 5]).slim_for_each(|x| result.push(x));

        future.await;

        assert_eq!(result, [2, 3, 5]);
    }

    #[tokio::test]
    async fn test_for_each_clone() {
        let result = Mutex::new(Vec::new());
        let future = stream::iter([2, 3, 5]).slim_for_each(|x| result.lock().unwrap().push(x));
        let future_2 = future.clone();

        future.await;

        assert_eq!(*result.lock().unwrap(), [2, 3, 5]);

        future_2.await;

        assert_eq!(*result.lock().unwrap(), [2, 3, 5, 2, 3, 5]);
    }
}
