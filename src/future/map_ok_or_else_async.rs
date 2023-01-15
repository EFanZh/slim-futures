#![allow(clippy::type_complexity)]

use crate::future::raw_map_ok_or_else_async::RawMapOkOrElseAsync;
use crate::support::{FnMut1, ResultFuture};
use core::future::{Future, IntoFuture};
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;
use futures_util::future::Either;

struct MapOkOrElseAsyncLeftFn<F, R> {
    f: F,
    phantom: PhantomData<R>,
}

impl<F, R> MapOkOrElseAsyncLeftFn<F, R> {
    fn new(f: F) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

impl<F, R> Clone for MapOkOrElseAsyncLeftFn<F, R>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            phantom: self.phantom,
        }
    }
}

impl<T, F, R> FnMut1<T> for MapOkOrElseAsyncLeftFn<F, R>
where
    F: FnMut1<T>,
    F::Output: IntoFuture,
{
    type Output = Either<<F::Output as IntoFuture>::IntoFuture, R>;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        Either::Left(self.f.call_mut(arg).into_future())
    }
}

struct MapOkOrElseAsyncRightFn<F, L> {
    f: F,
    phantom: PhantomData<L>,
}

impl<F, L> MapOkOrElseAsyncRightFn<F, L> {
    fn new(f: F) -> Self {
        Self {
            f,
            phantom: PhantomData,
        }
    }
}

impl<F, L> Clone for MapOkOrElseAsyncRightFn<F, L>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            phantom: self.phantom,
        }
    }
}

impl<T, F, L> FnMut1<T> for MapOkOrElseAsyncRightFn<F, L>
where
    F: FnMut1<T>,
    F::Output: IntoFuture,
{
    type Output = Either<L, <F::Output as IntoFuture>::IntoFuture>;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        Either::Right(self.f.call_mut(arg).into_future())
    }
}

pin_project_lite::pin_project! {
    pub struct MapOkOrElseAsync<Fut, D, F>
    where
        Fut: ResultFuture,
        D: FnMut1<Fut::Error>,
        D::Output: IntoFuture,
        F: FnMut1<Fut::Ok>,
        F::Output: IntoFuture<Output = <D::Output as IntoFuture>::Output>,
    {
        #[pin]
        inner: RawMapOkOrElseAsync<
            Fut,
            MapOkOrElseAsyncRightFn<D, <F::Output as IntoFuture>::IntoFuture>,
            MapOkOrElseAsyncLeftFn<F, <D::Output as IntoFuture>::IntoFuture>,
        >,
    }
}

impl<Fut, D, F> MapOkOrElseAsync<Fut, D, F>
where
    Fut: ResultFuture,
    D: FnMut1<Fut::Error>,
    D::Output: IntoFuture,
    F: FnMut1<Fut::Ok>,
    F::Output: IntoFuture<Output = <D::Output as IntoFuture>::Output>,
{
    pub(crate) fn new(fut: Fut, default: D, f: F) -> Self {
        Self {
            inner: RawMapOkOrElseAsync::new(
                fut,
                MapOkOrElseAsyncRightFn::new(default),
                MapOkOrElseAsyncLeftFn::new(f),
            ),
        }
    }
}

impl<Fut, D, F> Clone for MapOkOrElseAsync<Fut, D, F>
where
    Fut: ResultFuture + Clone,
    D: FnMut1<Fut::Error> + Clone,
    D::Output: IntoFuture,
    <D::Output as IntoFuture>::IntoFuture: Clone,
    F: FnMut1<Fut::Ok> + Clone,
    F::Output: IntoFuture<Output = <D::Output as IntoFuture>::Output>,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, D, F> Future for MapOkOrElseAsync<Fut, D, F>
where
    Fut: ResultFuture,
    D: FnMut1<Fut::Error>,
    D::Output: IntoFuture,
    F: FnMut1<Fut::Ok>,
    F::Output: IntoFuture<Output = <D::Output as IntoFuture>::Output>,
{
    type Output = <D::Output as IntoFuture>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, D, F> FusedFuture for MapOkOrElseAsync<Fut, D, F>
where
    Fut: ResultFuture + FusedFuture,
    D: FnMut1<Fut::Error>,
    D::Output: IntoFuture,
    <D::Output as IntoFuture>::IntoFuture: FusedFuture,
    F: FnMut1<Fut::Ok>,
    F::Output: IntoFuture<Output = <D::Output as IntoFuture>::Output>,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
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
        assert_eq!(future.by_ref().await, 6);
        assert!(future.is_terminated());
    }
}
