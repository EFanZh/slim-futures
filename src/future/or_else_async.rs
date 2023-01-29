use crate::future::map_err::MapErr;
use crate::future::try_flatten_err::TryFlattenErr;
use crate::support::{ResultFuture, Try};
use core::future::{Future, IntoFuture};
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct OrElseAsync<Fut, F>
    where
        Fut: ResultFuture,
        F: FnMut<(Fut::Error,)>,
        F::Output: IntoFuture,
    {
        #[pin]
        inner: TryFlattenErr<MapErr<Fut, F>>
    }
}

impl<Fut, F> OrElseAsync<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Error,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: TryFlattenErr::new(MapErr::new(fut, f)),
        }
    }
}

impl<Fut, F> Clone for OrElseAsync<Fut, F>
where
    Fut: ResultFuture + Clone,
    F: FnMut<(Fut::Error,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, F> Future for OrElseAsync<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Error,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = Fut::Ok>,
{
    type Output = <F::Output as IntoFuture>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for OrElseAsync<Fut, F>
where
    Fut: ResultFuture + FusedFuture,
    F: FnMut<(Fut::Error,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: Try<Output = Fut::Ok>,
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
    use futures_util::future::Ready;
    use futures_util::{future, TryFutureExt};
    use std::mem;
    use std::num::NonZeroU32;

    fn ok_plus_3(value: u32) -> Ready<Result<u32, u32>> {
        future::ok(value + 3)
    }

    fn err_plus_3(value: u32) -> Ready<Result<u32, u32>> {
        future::err(value + 3)
    }

    #[tokio::test]
    async fn test_or_else_async() {
        assert_eq!(future::ok::<u32, _>(2).slim_or_else_async(ok_plus_3).await, Ok(2));
        assert_eq!(future::ok::<u32, _>(2).slim_or_else_async(err_plus_3).await, Ok(2));
        assert_eq!(future::err::<u32, _>(2).slim_or_else_async(ok_plus_3).await, Ok(5));
        assert_eq!(future::err::<u32, _>(2).slim_or_else_async(err_plus_3).await, Err(5));
    }

    #[tokio::test]
    async fn test_or_else_async_with_option() {
        assert_eq!(
            future::err::<u32, _>(2).slim_or_else_async(|x| future::ready(Some(x + 3))).await,
            Some(5),
        );
    }

    #[tokio::test]
    async fn test_or_else_async_clone() {
        let future = future::err::<u32, _>(2).slim_or_else_async(err_plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, Err(5));
        assert_eq!(future_2.await, Err(5));
    }

    #[tokio::test]
    async fn test_or_else_async_fused_future() {
        let mut future = future::err::<u32, _>(2).slim_or_else_async(err_plus_3);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Err(5));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_or_else_async_is_slim() {
        let make_base_future = || crate::future::err_by_copy::<u32, _>(NonZeroU32::new(2).unwrap()).slim_map_err(drop);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_or_else_async(crate::future::err_by_copy);
        let future_2 = make_base_future().or_else(crate::future::err_by_copy);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, Err(()));
        assert_eq!(future_1.await, Err(()));
        assert_eq!(future_2.await, Err(()));
    }
}
