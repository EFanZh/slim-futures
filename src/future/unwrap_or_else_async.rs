use crate::future::map::Map;
use crate::support::{FnMut1, ResultFuture, TwoPhases};
use core::convert;
use core::future::Future;
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

#[derive(Clone)]
struct UnwrapOrElseAsyncFn<F> {
    inner: F,
}

impl<T, E, F> FnMut1<Result<T, E>> for UnwrapOrElseAsyncFn<F>
where
    F: FnMut1<E>,
{
    type Output = ControlFlow<T, F::Output>;

    fn call_mut(&mut self, arg: Result<T, E>) -> Self::Output {
        match arg {
            Ok(value) => ControlFlow::Break(value),
            Err(error) => ControlFlow::Continue(self.inner.call_mut(error)),
        }
    }
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct UnwrapOrElseAsync<Fut, F>
    where
        Fut: ResultFuture,
        F: FnMut1<Fut::Error>
    {
        #[pin]
        inner: TwoPhases<Map<Fut, UnwrapOrElseAsyncFn<F>>, F::Output>,
    }
}

impl<Fut, F> UnwrapOrElseAsync<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut1<Fut::Error>,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: TwoPhases::First {
                state: Map::new(fut, UnwrapOrElseAsyncFn { inner: f }),
            },
        }
    }
}

impl<Fut, F, T, E> Future for UnwrapOrElseAsync<Fut, F>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnMut1<E>,
    F::Output: Future<Output = T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll_with(cx, convert::identity, F::Output::poll)
    }
}

impl<Fut, F, T, E> FusedFuture for UnwrapOrElseAsync<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<E>,
    F::Output: FusedFuture<Output = T>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_future_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use crate::test_utilities::Yield;
    use futures_core::FusedFuture;
    use futures_util::future::{self, Ready};
    use futures_util::TryFutureExt;
    use std::mem;
    use std::num::NonZeroU32;

    fn plus_3(value: u32) -> Ready<u32> {
        future::ready(value + 3)
    }

    #[tokio::test]
    async fn test_unwrap_or_else_async() {
        assert_eq!(future::ok::<u32, _>(2).slim_unwrap_or_else_async(plus_3).await, 2);
        assert_eq!(future::err::<u32, _>(2).slim_unwrap_or_else_async(plus_3).await, 5);
    }

    #[tokio::test]
    async fn test_unwrap_or_else_async_with_pending() {
        let future = Yield::new(1)
            .slim_map(|()| Err::<u32, _>(()))
            .slim_unwrap_or_else_async(|()| future::ready(2));

        assert_eq!(future.await, 2);
    }

    #[tokio::test]
    async fn test_unwrap_or_else_async_clone() {
        let future = future::err::<u32, _>(2).slim_unwrap_or_else_async(plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, 5);
        assert_eq!(future_2.await, 5);
    }

    #[tokio::test]
    async fn test_unwrap_or_else_async_fused_future() {
        let mut future = future::err::<u32, _>(2).slim_unwrap_or_else_async(plus_3);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, 5);
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_or_else_async_is_slim() {
        let make_base_future = || crate::future::err::<u32, _>(NonZeroU32::new(2).unwrap()).slim_map_err(drop);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_or_else_async(crate::future::err);
        let future_2 = make_base_future().or_else(crate::future::err);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, Err(()));
        assert_eq!(future_1.await, Err(()));
        assert_eq!(future_2.await, Err(()));
    }
}
