use crate::try_future::TryFuture;
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

impl<Fut, T, E> TryFlatten<Fut>
where
    Fut: Future<Output = Result<T, E>>,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TryFlattenInner::First { fut },
        }
    }
}

impl<Fut, Fut2, E, U> Future for TryFlatten<Fut>
where
    Fut: Future<Output = Result<Fut2, E>>,
    Fut2: Future<Output = Result<U, E>>,
{
    type Output = Result<U, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.project().inner;

        loop {
            match inner.as_mut().project() {
                TryFlattenInnerProject::First { fut } => {
                    let fut = futures_core::ready!(fut.poll(cx)?);

                    inner.set(TryFlattenInner::Second { fut });
                }
                TryFlattenInnerProject::Second { fut } => return fut.poll(cx),
            }
        }
    }
}

impl<Fut, Fut2, E, U> FusedFuture for TryFlatten<Fut>
where
    Fut: FusedFuture<Output = Result<Fut2, E>>,
    Fut2: FusedFuture<Output = Result<U, E>>,
{
    fn is_terminated(&self) -> bool {
        match &self.inner {
            TryFlattenInner::First { fut } => fut.is_terminated(),
            TryFlattenInner::Second { fut } => fut.is_terminated(),
        }
    }
}
