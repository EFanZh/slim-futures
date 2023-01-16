use crate::async_iter::try_fold::TryFold;
use crate::support::{AsyncIterator, FnMut1, FnMut2, FusedAsyncIterator, Try};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

#[derive(Clone)]
struct TryForEachFn<F> {
    f: F,
}

impl<T, F> FnMut2<(), T> for TryForEachFn<F>
where
    F: FnMut1<T>,
{
    type Output = F::Output;

    fn call_mut(&mut self, (): (), arg_2: T) -> Self::Output {
        self.f.call_mut(arg_2)
    }
}

pin_project_lite::pin_project! {
    pub struct TryForEach<I, F> {
        #[pin]
        inner: TryFold<I, (), TryForEachFn<F>>,
    }
}

impl<I, F> TryForEach<I, F> {
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: TryFold::new(iter, (), TryForEachFn { f }),
        }
    }
}

impl<I, F> Clone for TryForEach<I, F>
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

impl<I, F> Future for TryForEach<I, F>
where
    I: AsyncIterator,
    F: FnMut1<I::Item>,
    F::Output: Try<Output = ()>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<I, F> FusedFuture for TryForEach<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut1<I::Item>,
    F::Output: Try<Output = ()>,
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
    async fn test_try_for_each() {
        let mut result = Vec::new();

        let future = stream::iter([2, 3, 5]).slim_try_for_each(|x| {
            result.push(x);

            Some(())
        });

        assert_eq!(future.await, Some(()));
        assert_eq!(result, [2, 3, 5]);
    }

    #[tokio::test]
    async fn test_try_for_each_fail() {
        let mut result = Vec::new();
        let future = stream::iter([2, 3, 5]).slim_try_for_each(|x| (x < 5).then(|| result.push(x)));

        assert_eq!(future.await, None);
        assert_eq!(result, [2, 3]);
    }

    #[tokio::test]
    async fn test_try_for_each_clone() {
        let result = Mutex::new(Vec::new());

        let future = stream::iter([2, 3, 5]).slim_try_for_each(|x| {
            result.lock().unwrap().push(x);

            Some(())
        });

        let future_2 = future.clone();

        assert_eq!(future.await, Some(()));
        assert_eq!(*result.lock().unwrap(), [2, 3, 5]);

        assert_eq!(future_2.await, Some(()));
        assert_eq!(*result.lock().unwrap(), [2, 3, 5, 2, 3, 5]);
    }
}
