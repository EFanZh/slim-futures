use crate::future::map_async::MapAsync;
use crate::support::fns::MapOkOrElseFn;
use crate::support::{FnMut1, ResultFuture};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct RawMapOkOrElseAsync<Fut, D, F>
    where
        Fut: ResultFuture,
        D: FnMut1<Fut::Error>,
        D::Output: IntoFuture,
        F: FnMut1<Fut::Ok, Output = D::Output>,
    {
        #[pin]
        inner: MapAsync<Fut, MapOkOrElseFn<D, F>>,
    }
}

impl<Fut, D, F> Clone for RawMapOkOrElseAsync<Fut, D, F>
where
    Fut: ResultFuture + Clone,
    D: FnMut1<Fut::Error> + Clone,
    D::Output: IntoFuture,
    <D::Output as IntoFuture>::IntoFuture: Clone,
    F: FnMut1<Fut::Ok, Output = D::Output> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, D, F> RawMapOkOrElseAsync<Fut, D, F>
where
    Fut: ResultFuture,
    D: FnMut1<Fut::Error>,
    D::Output: IntoFuture,
    F: FnMut1<Fut::Ok, Output = D::Output>,
{
    pub(crate) fn new(fut: Fut, default: D, f: F) -> Self {
        Self {
            inner: MapAsync::new(fut, MapOkOrElseFn::new(default, f)),
        }
    }
}

impl<Fut, D, F> Future for RawMapOkOrElseAsync<Fut, D, F>
where
    Fut: ResultFuture,
    D: FnMut1<Fut::Error>,
    D::Output: IntoFuture,
    F: FnMut1<Fut::Ok, Output = D::Output>,
{
    type Output = <D::Output as IntoFuture>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, D, F> FusedFuture for RawMapOkOrElseAsync<Fut, D, F>
where
    Fut: ResultFuture + FusedFuture,
    D: FnMut1<Fut::Error>,
    D::Output: IntoFuture,
    <D::Output as IntoFuture>::IntoFuture: FusedFuture,
    F: FnMut1<Fut::Ok, Output = D::Output>,
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
    use std::mem;
    use std::num::NonZeroU32;

    fn plus_3(value: u32) -> Ready<u32> {
        future::ready(value + 3)
    }

    fn plus_4(value: u32) -> Ready<u32> {
        future::ready(value + 4)
    }

    #[tokio::test]
    async fn test_raw_map_ok_or_else_async() {
        let future_1 = future::ok::<_, u32>(2).slim_raw_map_ok_or_else_async(plus_3, plus_4);
        let future_2 = future::err::<_, u32>(2).slim_raw_map_ok_or_else_async(plus_3, plus_4);

        assert_eq!(future_1.await, 6);
        assert_eq!(future_2.await, 5,);
    }

    #[tokio::test]
    async fn test_raw_map_ok_or_else_async_clone() {
        let future = future::ok::<_, u32>(2).slim_raw_map_ok_or_else_async(plus_3, plus_4);
        let future_2 = future.clone();

        assert_eq!(future.await, 6);
        assert_eq!(future_2.await, 6);
    }

    #[tokio::test]
    async fn test_raw_map_ok_or_else_async_fused_future() {
        let mut future = future::ok::<_, u32>(2).slim_raw_map_ok_or_else_async(plus_3, plus_4);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, 6);
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_raw_map_ok_or_else_async_is_slim() {
        let make_base_future = || crate::future::ok::<_, NonZeroU32>(NonZeroU32::new(2).unwrap());
        let base_future = make_base_future();
        let f = |_| crate::future::lazy(|_| 3);
        let future = make_base_future().slim_raw_map_ok_or_else_async(f, f);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future));
        assert_eq!(base_future.await.map(NonZeroU32::get), Ok(2));
        assert_eq!(future.await, 3);
    }
}
