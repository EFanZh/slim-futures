use crate::support::TryFuture;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    #[project = TryFlattenErrInnerProject]
    enum TryFlattenErrInner<Fut>
    where
        Fut: TryFuture,
    {
        First {
            #[pin]
            fut: Fut,
        },
        Second {
            #[pin]
            fut: Fut::Error,
        },
    }
}

pin_project_lite::pin_project! {
    pub struct TryFlattenErr<Fut>
    where
        Fut: TryFuture,
    {
        #[pin]
        inner: TryFlattenErrInner<Fut>,
    }
}

impl<Fut, Fut2, T> TryFlattenErr<Fut>
where
    Fut: Future<Output = Result<T, Fut2>>,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TryFlattenErrInner::First { fut },
        }
    }
}

impl<Fut, Fut2, T> Clone for TryFlattenErr<Fut>
where
    Fut: Clone + Future<Output = Result<T, Fut2>>,
    Fut2: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: match &self.inner {
                TryFlattenErrInner::First { fut } => TryFlattenErrInner::First { fut: fut.clone() },
                TryFlattenErrInner::Second { fut } => TryFlattenErrInner::Second { fut: fut.clone() },
            },
        }
    }
}

impl<Fut, Fut2, T, E> Future for TryFlattenErr<Fut>
where
    Fut: Future<Output = Result<T, Fut2>>,
    Fut2: Future<Output = Result<T, E>>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.project().inner;

        if let TryFlattenErrInnerProject::First { fut } = inner.as_mut().project() {
            let fut = match futures_core::ready!(fut.poll(cx)) {
                Ok(value) => return Poll::Ready(Ok(value)),
                Err(fut) => fut,
            };

            inner.set(TryFlattenErrInner::Second { fut });
        }

        if let TryFlattenErrInnerProject::Second { fut } = inner.project() {
            fut.poll(cx)
        } else {
            unreachable!() // TODO: Is `unreachable_unchecked()` necessary for compiler to optimize away this branch?
        }
    }
}

impl<Fut, Fut2, T, E> FusedFuture for TryFlattenErr<Fut>
where
    Fut: FusedFuture<Output = Result<T, Fut2>>,
    Fut2: FusedFuture<Output = Result<T, E>>,
{
    fn is_terminated(&self) -> bool {
        match &self.inner {
            TryFlattenErrInner::First { fut } => fut.is_terminated(),
            TryFlattenErrInner::Second { fut } => fut.is_terminated(),
        }
    }
}
