use crate::support::TryFuture;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    #[project = TryFlattenInnerProject]
    enum TryFlattenInner<Fut>
    where
        Fut: TryFuture,
    {
        First {
            #[pin]
            fut: Fut,
        },
        Second {
            #[pin]
            fut: Fut::Ok,
        },
    }
}

pin_project_lite::pin_project! {
    pub struct TryFlatten<Fut>
    where
        Fut: TryFuture,
    {
        #[pin]
        inner: TryFlattenInner<Fut>,
    }
}

impl<Fut, Fut2, E> TryFlatten<Fut>
where
    Fut: Future<Output = Result<Fut2, E>>,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TryFlattenInner::First { fut },
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
            inner: match &self.inner {
                TryFlattenInner::First { fut } => TryFlattenInner::First { fut: fut.clone() },
                TryFlattenInner::Second { fut } => TryFlattenInner::Second { fut: fut.clone() },
            },
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
        let mut inner = self.project().inner;

        if let TryFlattenInnerProject::First { fut } = inner.as_mut().project() {
            let fut = futures_core::ready!(fut.poll(cx))?;

            inner.set(TryFlattenInner::Second { fut });
        }

        if let TryFlattenInnerProject::Second { fut } = inner.project() {
            fut.poll(cx)
        } else {
            unreachable!() // TODO: Is `unreachable_unchecked()` necessary for compiler to optimize away this branch?
        }
    }
}

impl<Fut, Fut2, E, T> FusedFuture for TryFlatten<Fut>
where
    Fut: FusedFuture<Output = Result<Fut2, E>>,
    Fut2: FusedFuture<Output = Result<T, E>>,
{
    fn is_terminated(&self) -> bool {
        match &self.inner {
            TryFlattenInner::First { fut } => fut.is_terminated(),
            TryFlattenInner::Second { fut } => fut.is_terminated(),
        }
    }
}
