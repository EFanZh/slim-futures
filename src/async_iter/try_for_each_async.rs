use crate::async_iter::try_fold_async::TryFoldAsync;
use crate::support::fns::ForEachFn;
use crate::support::{AsyncIterator, Try};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::CopyFn;
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct TryForEachAsync<I, F>
    where
        I: AsyncIterator,
        F: FnMut<(I::Item,)>,
        F::Output: IntoFuture,
        <F::Output as IntoFuture>::Output: Try,
    {
        #[pin]
        inner: TryFoldAsync<I, (), CopyFn, ForEachFn<F>>,
    }
}

impl<I, F> TryForEachAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = ()>,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: TryFoldAsync::new(iter, (), CopyFn::default(), ForEachFn::new(f)),
        }
    }
}

impl<I, F> Clone for TryForEachAsync<I, F>
where
    I: AsyncIterator + Clone,
    F: FnMut<(I::Item,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = ()>,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F> Future for TryForEachAsync<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = ()>,
{
    type Output = <F::Output as IntoFuture>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use core::future;
    use futures_util::stream;
    use std::sync::Mutex;
    use std::vec::Vec;

    #[tokio::test]
    async fn test_try_for_each_async() {
        let mut result = Vec::new();

        let future = stream::iter([2, 3, 5]).slim_try_for_each_async(|x| {
            result.push(x);

            future::ready(Some(()))
        });

        assert_eq!(future.await, Some(()));
        assert_eq!(result, [2, 3, 5]);
    }

    #[tokio::test]
    async fn test_try_for_each_async_fail() {
        let mut result = Vec::new();

        let future =
            stream::iter([2, 3, 5]).slim_try_for_each_async(|x| future::ready((x < 5).then(|| result.push(x))));

        assert_eq!(future.await, None);
        assert_eq!(result, [2, 3]);
    }

    #[tokio::test]
    async fn test_try_for_each_async_clone() {
        let result = Mutex::new(Vec::new());

        let future = stream::iter([2, 3, 5]).slim_try_for_each_async(|x| {
            result.lock().unwrap().push(x);

            future::ready(Some(()))
        });

        let future_2 = future.clone();

        assert_eq!(future.await, Some(()));
        assert_eq!(*result.lock().unwrap(), [2, 3, 5]);

        assert_eq!(future_2.await, Some(()));
        assert_eq!(*result.lock().unwrap(), [2, 3, 5, 2, 3, 5]);
    }
}
