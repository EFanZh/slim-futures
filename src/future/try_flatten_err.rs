use crate::support::states::TwoPhases;
use crate::support::{ResultFuture, Try};
use core::future::{Future, IntoFuture};
use core::ops::ControlFlow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::FusedFuture;

pin_project_lite::pin_project! {
    pub struct TryFlattenErr<Fut>
    where
        Fut: ResultFuture,
        Fut::Error: IntoFuture,
    {
        #[pin]
        inner: TwoPhases<Fut, <Fut::Error as IntoFuture>::IntoFuture>,
    }
}

impl<Fut> TryFlattenErr<Fut>
where
    Fut: ResultFuture,
    Fut::Error: IntoFuture,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TwoPhases::new(fut),
        }
    }
}

impl<Fut> Clone for TryFlattenErr<Fut>
where
    Fut: ResultFuture + Clone,
    Fut::Error: IntoFuture,
    <Fut::Error as IntoFuture>::IntoFuture: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut> Future for TryFlattenErr<Fut>
where
    Fut: ResultFuture,
    Fut::Error: IntoFuture,
    <Fut::Error as IntoFuture>::Output: Try<Output = Fut::Ok>,
{
    type Output = <Fut::Error as IntoFuture>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        fn dispatch<T, E>(result: Result<T, E>) -> ControlFlow<E::Output, E::IntoFuture>
        where
            E: IntoFuture,
            <E as IntoFuture>::Output: Try<Output = T>,
        {
            match result {
                Ok(value) => ControlFlow::Break(<E as IntoFuture>::Output::from_output(value)),
                Err(error) => ControlFlow::Continue(error.into_future()),
            }
        }

        self.project()
            .inner
            .poll_with(cx, dispatch, <Fut::Error as IntoFuture>::IntoFuture::poll)
    }
}

impl<Fut> FusedFuture for TryFlattenErr<Fut>
where
    Fut: ResultFuture + FusedFuture,
    Fut::Error: IntoFuture,
    <Fut::Error as IntoFuture>::Output: Try<Output = Fut::Ok>,
    <Fut::Error as IntoFuture>::IntoFuture: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_future_terminated()
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use crate::future::{err, ok, ready};
    use crate::test_utilities::Yield;
    use futures_core::FusedFuture;
    use futures_util::future::Ready;
    use std::mem;
    use std::num::NonZeroU32;

    #[tokio::test]
    async fn test_try_flatten_err() {
        let future_1 = ok::ok_by_copy::<u32, Ready<Result<_, u32>>>(2).slim_try_flatten_err();
        let future_2 = err::err_by_clone::<u32, _>(ok::ok_by_copy::<_, u32>(2)).slim_try_flatten_err();
        let future_3 = err::err_by_clone::<u32, _>(err::err_by_copy::<_, u32>(2)).slim_try_flatten_err();

        assert_eq!(future_1.await, Ok(2));
        assert_eq!(future_2.await, Ok(2));
        assert_eq!(future_3.await, Err(2));
    }

    #[tokio::test]
    async fn test_try_flatten_err_with_option() {
        let future = err::err_by_clone::<u32, _>(ready::ready_by_copy(Some(2))).slim_try_flatten_err();

        assert_eq!(future.await, Some(2));
    }

    #[tokio::test]
    async fn test_try_flatten_err_with_pending() {
        let future = Yield::new(1)
            .slim_map(|()| Err::<u32, _>(err::err_by_copy::<_, u32>(2)))
            .slim_try_flatten_err();

        assert_eq!(future.await, Err(2));
    }

    #[tokio::test]
    async fn test_try_flatten_err_clone() {
        let future = err::err_by_clone::<u32, _>(err::err_by_copy::<_, u32>(2)).slim_try_flatten_err();
        let future_2 = future.clone();

        assert_eq!(future.await, Err(2));
        assert_eq!(future_2.await, Err(2));
    }

    #[tokio::test]
    async fn test_try_flatten_err_fused_future() {
        let mut future =
            futures_util::future::err::<u32, _>(futures_util::future::err::<_, u32>(2)).slim_try_flatten_err();

        assert!(!future.is_terminated());
        assert_eq!(future.by_ref().await, Err(2));
        assert!(future.is_terminated());
    }

    #[tokio::test]
    async fn test_try_flatten_err_is_slim() {
        let make_base_future =
            || err::err_by_copy(NonZeroU32::new(2).unwrap()).slim_map_err(|_| err::err_by_copy::<(), _>(()));

        let base_future = make_base_future();
        let future = make_base_future().slim_try_flatten_err();

        assert_eq!(mem::size_of_val(&base_future), mem::size_of_val(&future));
        assert_eq!(base_future.await.unwrap_err().await, Err(()));
        assert_eq!(future.await, Err(()));
    }
}
