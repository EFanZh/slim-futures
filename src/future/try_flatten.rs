use crate::support::{TryFuture, TwoPhases};
use futures_core::FusedFuture;
use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct TryFlatten<Fut>
    where
        Fut: TryFuture,
    {
        #[pin]
        inner: TwoPhases<Fut, Fut::Ok>,
    }
}

impl<Fut, Fut2, E> TryFlatten<Fut>
where
    Fut: Future<Output = Result<Fut2, E>>,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TwoPhases::First { state: fut },
        }
    }
}

impl<Fut, Fut2, E> Clone for TryFlatten<Fut>
where
    Fut: Clone + Future<Output = Result<Fut2, E>>,
    Fut2: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Fut, Fut2, E, T> Future for TryFlatten<Fut>
where
    Fut: Future<Output = Result<Fut2, E>>,
    Fut2: Future<Output = Result<T, E>>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll_with(
            cx,
            |fut, cx| match fut.poll(cx) {
                Poll::Pending => ControlFlow::Break(Poll::Pending),
                Poll::Ready(Ok(fut)) => ControlFlow::Continue(fut),
                Poll::Ready(Err(error)) => ControlFlow::Break(Poll::Ready(Err(error))),
            },
            Fut2::poll,
        )
    }
}

impl<Fut, Fut2, E, T> FusedFuture for TryFlatten<Fut>
where
    Fut: FusedFuture<Output = Result<Fut2, E>>,
    Fut2: FusedFuture<Output = Result<T, E>>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_future_terminated()
    }
}
