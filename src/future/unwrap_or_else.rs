use crate::future::map::Map;
use crate::support::ResultFuture;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

#[derive(Clone)]
struct UnwrapOrElseFn<F>
where
    F: ?Sized,
{
    f: F,
}

impl<T, E, F> FnMut<(Result<T, E>,)> for UnwrapOrElseFn<F>
where
    F: FnMut<(E,), Output = T> + ?Sized,
{
    type Output = T;

    fn call_mut(&mut self, args: (Result<T, E>,)) -> Self::Output {
        args.0.unwrap_or_else(|error| self.f.call_mut((error,)))
    }
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct UnwrapOrElse<Fut, F>
    where
        F: ?Sized,
    {
        #[pin]
        inner: Map<Fut, UnwrapOrElseFn<F>>,
    }
}

impl<Fut, F> UnwrapOrElse<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Map::new(fut, UnwrapOrElseFn { f }),
        }
    }
}

impl<Fut, F> Future for UnwrapOrElse<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Error,), Output = Fut::Ok> + ?Sized,
{
    type Output = Fut::Ok;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for UnwrapOrElse<Fut, F>
where
    Fut: ResultFuture + FusedFuture,
    F: FnMut<(Fut::Error,), Output = Fut::Ok> + ?Sized,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::err;
    use crate::future::future_ext::FutureExt;
    use futures_core::FusedFuture;
    use futures_util::future;
    use std::mem;

    fn plus_3(value: u32) -> u32 {
        value + 3
    }

    #[tokio::test]
    async fn test_unwrap_or_else() {
        assert_eq!(future::ok::<u32, _>(2).slim_unwrap_or_else(plus_3).await, 2);
        assert_eq!(future::err::<u32, _>(2).slim_unwrap_or_else(plus_3).await, 5);
    }

    #[tokio::test]
    async fn test_unwrap_or_else_clone() {
        let future = future::err::<u32, _>(2).slim_unwrap_or_else(plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, 5);
        assert_eq!(future_2.await, 5);
    }

    #[tokio::test]
    async fn test_unwrap_or_else_fused_future() {
        let mut future = future::err::<u32, _>(2).slim_unwrap_or_else(plus_3);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, 5);
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_unwrap_or_else_is_slim() {
        let make_base_future = || err::err_by_copy::<u32, u32>(2);
        let base_future = make_base_future();
        let future = make_base_future().slim_or_else(Err);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future));
        assert_eq!(base_future.await, Err(2));
        assert_eq!(future.await, Err(2));
    }
}
