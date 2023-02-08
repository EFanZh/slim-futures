use crate::future::map::Map;
use crate::support::states::TwoPhases;
use crate::support::ResultFuture;
use core::future::{Future, IntoFuture};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

#[derive(Clone)]
struct UnwrapOrElseAsyncFn<F> {
    f: F,
}

impl<T, E, F> FnMut<(Result<T, E>,)> for UnwrapOrElseAsyncFn<F>
where
    F: FnMut<(E,)>,
{
    type Output = ControlFlow<T, F::Output>;

    fn call_mut(&mut self, args: (Result<T, E>,)) -> Self::Output {
        match args.0 {
            Ok(value) => ControlFlow::Break(value),
            Err(error) => ControlFlow::Continue(self.f.call_mut((error,))),
        }
    }
}

pin_project_lite::pin_project! {
    pub struct UnwrapOrElseAsync<Fut, F>
    where
        Fut: ResultFuture,
        F: FnMut<(Fut::Error,)>,
        F::Output: IntoFuture,
    {
        #[pin]
        inner: TwoPhases<Map<Fut, UnwrapOrElseAsyncFn<F>>, <F::Output as IntoFuture>::IntoFuture>,
    }
}

impl<Fut, F> UnwrapOrElseAsync<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Error,)>,
    F::Output: IntoFuture,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: TwoPhases::new(Map::new(fut, UnwrapOrElseAsyncFn { f })),
        }
    }
}

impl<Fut, F> Clone for UnwrapOrElseAsync<Fut, F>
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

impl<Fut, F> Future for UnwrapOrElseAsync<Fut, F>
where
    Fut: ResultFuture,
    F: FnMut<(Fut::Error,)>,
    F::Output: IntoFuture<Output = Fut::Ok>,
{
    type Output = Fut::Ok;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        fn dispatch<B, C>(result: ControlFlow<B, C>) -> ControlFlow<B, C::IntoFuture>
        where
            C: IntoFuture,
        {
            match result {
                ControlFlow::Continue(output) => ControlFlow::Continue(output.into_future()),
                ControlFlow::Break(residual) => ControlFlow::Break(residual),
            }
        }

        self.project()
            .inner
            .poll_with(cx, dispatch, <F::Output as IntoFuture>::IntoFuture::poll)
    }
}

impl<Fut, F> FusedFuture for UnwrapOrElseAsync<Fut, F>
where
    Fut: ResultFuture + FusedFuture,
    F: FnMut<(Fut::Error,)>,
    F::Output: IntoFuture<Output = Fut::Ok>,
    <F::Output as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_future_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::err;
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
    async fn test_unwrap_or_else_async_is_slim() {
        let make_base_future = || err::err_by_copy::<u32, _>(NonZeroU32::new(2).unwrap()).slim_map_err(drop);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_or_else_async(err::err_by_copy);
        let future_2 = make_base_future().or_else(err::err_by_copy);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, Err(()));
        assert_eq!(future_1.await, Err(()));
        assert_eq!(future_2.await, Err(()));
    }
}
