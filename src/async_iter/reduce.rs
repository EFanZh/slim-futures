use crate::async_iter::Fold;
use crate::support::{AsyncIterator, FusedAsyncIterator};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::MemTakeFn;
use fn_traits::FnMut;
use futures_core::FusedFuture;

#[derive(Clone)]
struct ReduceFn<F> {
    f: F,
}

impl<T, F> FnMut<(Option<T>, T)> for ReduceFn<F>
where
    F: FnMut<(T, T), Output = T>,
{
    type Output = Option<T>;

    fn call_mut(&mut self, args: (Option<T>, T)) -> Self::Output {
        Some(if let Some(acc) = args.0 {
            self.f.call_mut((acc, args.1))
        } else {
            args.1
        })
    }
}

pin_project_lite::pin_project! {
    pub struct Reduce<I, F>
    where
        I: AsyncIterator,
    {
        #[pin]
        inner: Fold<I, Option<I::Item>, MemTakeFn, ReduceFn<F>>,
    }
}

impl<I, F> Reduce<I, F>
where
    I: AsyncIterator,
{
    pub(crate) fn new(iter: I, f: F) -> Self {
        Self {
            inner: Fold::new(iter, None, MemTakeFn::default(), ReduceFn { f }),
        }
    }
}

impl<I, F> Clone for Reduce<I, F>
where
    I: AsyncIterator + Clone,
    I::Item: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<I, F> Future for Reduce<I, F>
where
    I: AsyncIterator,
    F: FnMut<(I::Item, I::Item), Output = I::Item>,
{
    type Output = Option<I::Item>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<I, F> FusedFuture for Reduce<I, F>
where
    I: FusedAsyncIterator,
    F: FnMut<(I::Item, I::Item), Output = I::Item>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::async_iter::async_iter_ext::AsyncIteratorExt;
    use futures_util::stream;

    fn add(lhs: u32, rhs: u32) -> u32 {
        lhs + rhs
    }

    #[tokio::test]
    async fn test_reduce() {
        let future = stream::iter([2, 3, 5]).slim_reduce(add);

        assert_eq!(future.await, Some(10));
    }

    #[tokio::test]
    async fn test_reduce_empty() {
        let future = stream::iter(None::<u32>).slim_reduce(add);

        assert_eq!(future.await, None);
    }

    #[tokio::test]
    async fn test_reduce_clone() {
        let future = stream::iter([2, 3, 5]).slim_reduce(add);
        let future_2 = future.clone();

        assert_eq!(future.await, Some(10));
        assert_eq!(future_2.await, Some(10));
    }
}
