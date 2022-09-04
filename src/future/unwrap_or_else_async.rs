use crate::support::{FnMut1, PinnedAndNotPinned, TryFuture, TwoPhases};
use futures_core::FusedFuture;
use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
struct UnwrapOrElseFn<F> {
    inner: F,
}

impl<T, E, F> FnMut1<Result<T, E>> for UnwrapOrElseFn<F>
where
    F: FnMut1<E, Output = T>,
{
    type Output = T;

    fn call_mut(&mut self, arg: Result<T, E>) -> Self::Output {
        arg.unwrap_or_else(|error| self.inner.call_mut(error))
    }
}

pin_project_lite::pin_project! {
    #[derive(Clone)]
    pub struct UnwrapOrElseAsync<Fut, F>
    where
        Fut: TryFuture,
        F: FnMut1<Fut::Error>
    {
        #[pin]
        inner: TwoPhases<PinnedAndNotPinned<Fut, F>, F::Output>,
    }
}

impl<Fut, F> UnwrapOrElseAsync<Fut, F>
where
    Fut: TryFuture,
    F: FnMut1<Fut::Error>,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: TwoPhases::First {
                state: PinnedAndNotPinned::new(fut, f),
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
        self.project().inner.poll_with(
            cx,
            |state, cx| {
                let state = state.project();

                match state.pinned.poll(cx) {
                    Poll::Pending => ControlFlow::Break(Poll::Pending),
                    Poll::Ready(Ok(value)) => ControlFlow::Break(Poll::Ready(value)),
                    Poll::Ready(Err(error)) => ControlFlow::Continue(state.not_pinned.call_mut(error)),
                }
            },
            F::Output::poll,
        )
    }
}

impl<Fut, F, T, E> FusedFuture for UnwrapOrElseAsync<Fut, F>
where
    Fut: FusedFuture<Output = Result<T, E>>,
    F: FnMut1<E>,
    F::Output: FusedFuture<Output = T>,
{
    fn is_terminated(&self) -> bool {
        match &self.inner {
            TwoPhases::First { state } => state.pinned.is_terminated(),
            TwoPhases::Second { state } => state.is_terminated(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::future::future_ext::FutureExt;
    use futures_core::FusedFuture;
    use futures_util::future;

    #[tokio::test]
    async fn test_unwrap_or_else_async() {
        assert_eq!(
            future::ready(Ok::<u32, u32>(2))
                .slim_unwrap_or_else_async(|value| future::ready(value + 3))
                .await,
            2,
        );

        assert_eq!(
            future::ready(Err::<u32, u32>(2))
                .slim_unwrap_or_else_async(|value| future::ready(value + 3))
                .await,
            5,
        );
    }

    #[tokio::test]
    async fn test_unwrap_or_else_async_clone() {
        let future = future::ready(Err::<u32, u32>(2)).slim_unwrap_or_else_async(|value| future::ready(value + 3));
        let future_2 = future.clone();

        assert_eq!(future.await, 5);
        assert_eq!(future_2.await, 5);
    }

    #[tokio::test]
    async fn test_unwrap_or_else_async_fused_future() {
        let mut future = future::ready(Err::<u32, u32>(2)).slim_unwrap_or_else_async(|value| future::ready(value + 3));

        assert!(!future.is_terminated());
        assert_eq!((&mut future).await, 5);
        assert!(future.is_terminated());
    }
}
