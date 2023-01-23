use crate::future::map::Map;
use crate::future::try_flatten::TryFlatten;
use crate::support::{FromResidual, RawResidual, Try};
use core::future::{Future, IntoFuture};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;
use futures_core::FusedFuture;

#[derive(Clone)]
struct AndThenAsyncFn<F> {
    inner: F,
}

impl<T, F> FnMut<(T,)> for AndThenAsyncFn<F>
where
    T: Try,
    F: FnMut<(T::Output,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: FromResidual<T::Residual>,
{
    type Output = RawResidual<T::Residual, F::Output>;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        match args.0.branch() {
            ControlFlow::Continue(output) => RawResidual::from_output(self.inner.call_mut((output,))),
            ControlFlow::Break(residual) => RawResidual::from_residual(residual),
        }
    }
}

pin_project_lite::pin_project! {
    pub struct AndThenAsync<Fut, F>
    where
        Fut: Future,
        Fut::Output: Try,
        F: FnMut<(<Fut::Output as Try>::Output,)>,
        F::Output: IntoFuture,
        <F::Output as IntoFuture>::Output: FromResidual<<Fut::Output as Try>::Residual>,
        <F::Output as IntoFuture>::Output: Try,
    {
        #[pin]
        inner: TryFlatten<Map<Fut, AndThenAsyncFn<F>>>
    }
}

impl<Fut, F> AndThenAsync<Fut, F>
where
    Fut: Future,
    Fut::Output: Try,
    F: FnMut<(<Fut::Output as Try>::Output,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: FromResidual<<Fut::Output as Try>::Residual> + Try,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: TryFlatten::new(Map::new(fut, AndThenAsyncFn { inner: f })),
        }
    }
}

impl<Fut, F> Clone for AndThenAsync<Fut, F>
where
    Fut: Future + Clone,
    Fut::Output: Try,
    F: FnMut<(<Fut::Output as Try>::Output,)> + Clone,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: FromResidual<<Fut::Output as Try>::Residual> + Try,
    <F::Output as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, F> Future for AndThenAsync<Fut, F>
where
    Fut: Future,
    Fut::Output: Try,
    F: FnMut<(<Fut::Output as Try>::Output,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: FromResidual<<Fut::Output as Try>::Residual> + Try,
{
    type Output = <F::Output as IntoFuture>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for AndThenAsync<Fut, F>
where
    Fut: FusedFuture,
    Fut::Output: Try,
    F: FnMut<(<Fut::Output as Try>::Output,)>,
    F::Output: IntoFuture,
    <F::Output as IntoFuture>::Output: FromResidual<<Fut::Output as Try>::Residual> + Try,
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
    use futures_util::future::{self, Ready, TryFutureExt};
    use std::mem;
    use std::num::NonZeroU32;

    fn ok_plus_3(value: u32) -> Ready<Result<u32, u32>> {
        future::ok(value + 3)
    }

    fn err_plus_3(value: u32) -> Ready<Result<u32, u32>> {
        future::err(value + 3)
    }

    #[tokio::test]
    async fn test_and_then_async() {
        assert_eq!(future::ok::<_, u32>(2).slim_and_then_async(ok_plus_3).await, Ok(5));
        assert_eq!(future::ok::<_, u32>(2).slim_and_then_async(err_plus_3).await, Err(5));
        assert_eq!(future::err::<_, u32>(2).slim_and_then_async(ok_plus_3).await, Err(2));
        assert_eq!(future::err::<_, u32>(2).slim_and_then_async(err_plus_3).await, Err(2));
    }

    #[tokio::test]
    async fn test_and_then_async_clone() {
        let future = future::ok::<u32, u32>(2).slim_and_then_async(ok_plus_3);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(5));
        assert_eq!(future_2.await, Ok(5));
    }

    #[tokio::test]
    async fn test_and_then_async_fused_future() {
        let mut future = future::ok::<u32, u32>(2).slim_and_then_async(ok_plus_3);

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Ok(5));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_and_then_async_is_slim() {
        let make_base_future = || crate::future::ok_by_copy::<_, u32>(NonZeroU32::new(2).unwrap()).slim_map_ok(drop);
        let base_future = make_base_future();
        let future_1 = make_base_future().slim_and_then_async(crate::future::ok_by_copy::<_, u32>);
        let future_2 = make_base_future().and_then(crate::future::ok_by_copy);

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(base_future.await, Ok(()));
        assert_eq!(future_1.await, Ok(()));
        assert_eq!(future_2.await, Ok(()));
    }
}
