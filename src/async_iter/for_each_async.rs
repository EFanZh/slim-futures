use crate::async_iter::fold_async::FoldAsync;
use crate::support::{AsyncIterator, FnMut1, FnMut2, FusedAsyncIterator};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

#[derive(Clone)]
struct ForEachAsyncFn<F> {
    f: F,
}

impl<T, F> FnMut2<(), T> for ForEachAsyncFn<F>
where
    F: FnMut1<T>,
{
    type Output = F::Output;

    fn call_mut(&mut self, (): (), arg_2: T) -> Self::Output {
        self.f.call_mut(arg_2)
    }
}

pin_project_lite::pin_project! {
    pub struct ForEachAsync<I, F>
    where
        I: AsyncIterator,
        F: FnMut1<I::Item>,
        F::Output: IntoFuture,
    {
        #[pin]
        inner: FoldAsync<I, (), ForEachAsyncFn<F>>,
    }
}

impl<I, F> ForEachAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: FoldAsync::new(iter, (), ForEachAsyncFn { f }),
        }
    }
}

impl<I, F> Clone for ForEachAsync<I, F>
where
    I: AsyncIterator + Clone,
    F: FnMut1<I::Item> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F> Future for ForEachAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item>,
    F::Output: IntoFuture<Output = ()>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<I, F> FusedFuture for ForEachAsync<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut1<I::Item>,
    F::Output: IntoFuture<Output = ()>,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::{future, stream};
    use std::sync::Mutex;
    use std::vec::Vec;

    #[tokio::test]
    async fn test_for_each_async() {
        let mut result = Vec::new();

        let future = stream::iter([2, 3, 5]).slim_for_each_async(|x| {
            result.push(x);

            future::ready(())
        });

        future.await;

        assert_eq!(result, [2, 3, 5]);
    }

    #[tokio::test]
    async fn test_for_each_async_clone() {
        let result = Mutex::new(Vec::new());

        let future = stream::iter([2, 3, 5]).slim_for_each_async(|x| {
            result.lock().unwrap().push(x);

            future::ready(())
        });

        let future_2 = future.clone();

        future.await;

        assert_eq!(*result.lock().unwrap(), [2, 3, 5]);

        future_2.await;

        assert_eq!(*result.lock().unwrap(), [2, 3, 5, 2, 3, 5]);
    }
}
