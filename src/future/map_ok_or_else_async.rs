use crate::future::raw_map_ok_or_else_async::RawMapOkOrElseAsync;
use crate::support::fns::{ComposeFn, EitherLeftFn, EitherRightFn};
use crate::support::{FnMut1, TryFuture};
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

type ExtractOk<Fut> = <Fut as TryFuture>::Ok;
type ExtractError<Fut> = <Fut as TryFuture>::Error;

type MapOkOrElseOkFn<Fut, D, F> =
    ComposeFn<F, EitherLeftFn<<F as FnMut1<ExtractOk<Fut>>>::Output, <D as FnMut1<ExtractError<Fut>>>::Output>>;

type MapOkOrElseErrFn<Fut, D, F> =
    ComposeFn<D, EitherRightFn<<F as FnMut1<ExtractOk<Fut>>>::Output, <D as FnMut1<ExtractError<Fut>>>::Output>>;

pin_project_lite::pin_project! {
    pub struct MapOkOrElseAsync<Fut, D, F>
    where
        Fut: TryFuture,
        D: FnMut1<Fut::Error>,
        F: FnMut1<Fut::Ok>,
    {
        #[pin]
        inner: RawMapOkOrElseAsync<Fut, MapOkOrElseErrFn<Fut, D, F>, MapOkOrElseOkFn<Fut, D, F>>,
    }
}

impl<Fut, D, F, T, E> Clone for MapOkOrElseAsync<Fut, D, F>
where
    Fut: Future<Output = Result<T, E>> + Clone,
    D: FnMut1<E> + Clone,
    F: FnMut1<T> + Clone,
    D::Output: Clone,
    F::Output: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, D, F, T, E> MapOkOrElseAsync<Fut, D, F>
where
    Fut: Future<Output = Result<T, E>>,
    D: FnMut1<E>,
    F: FnMut1<T>,
{
    pub(crate) fn new(fut: Fut, default: D, f: F) -> Self {
        Self {
            inner: RawMapOkOrElseAsync::new(
                fut,
                ComposeFn::new(default, EitherRightFn::default()),
                ComposeFn::new(f, EitherLeftFn::default()),
            ),
        }
    }
}

impl<Fut, D, F, T, E> Future for MapOkOrElseAsync<Fut, D, F>
where
    Fut: Future<Output = Result<T, E>>,
    D: FnMut1<E>,
    F: FnMut1<T>,
    D::Output: Future,
    F::Output: Future<Output = <D::Output as Future>::Output>,
{
    type Output = <D::Output as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, D, F, T, E> FusedFuture for MapOkOrElseAsync<Fut, D, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    D: FnMut1<E>,
    F: FnMut1<T>,
    D::Output: FusedFuture,
    F::Output: FusedFuture<Output = <D::Output as Future>::Output>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use futures_core::FusedFuture;
    use futures_util::future::{self, Ready};

    fn plus_3(value: u32) -> Ready<u32> {
        future::ready(value + 3)
    }

    fn plus_4(value: u32) -> Ready<u32> {
        future::ready(value + 4)
    }

    #[tokio::test]
    async fn test_map_ok_or_else_async() {
        assert_eq!(
            future::ok::<_, u32>(2).slim_map_ok_or_else_async(plus_3, plus_4).await,
            6,
        );

        assert_eq!(
            future::err::<_, u32>(2).slim_map_ok_or_else_async(plus_3, plus_4).await,
            5,
        );
    }

    #[tokio::test]
    async fn test_map_ok_or_else_async_clone() {
        let future = future::ok::<_, u32>(2).slim_map_ok_or_else_async(plus_3, plus_4);
        let future_2 = future.clone();

        assert_eq!(future.await, 6);
        assert_eq!(future_2.await, 6);
    }

    #[tokio::test]
    async fn test_map_ok_or_else_async_fused_future() {
        let mut future = future::ok::<_, u32>(2).slim_map_ok_or_else_async(plus_3, plus_4);

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, 6);
        assert!(future.is_terminated());
    }
}
