use futures::future::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait TryFuture: Future<Output = Result<Self::Ok, Self::Error>> {
    type Ok;
    type Error;
}

impl<Fut, T, E> TryFuture for Fut
where
    Fut: Future<Output = Result<T, E>>,
{
    type Ok = T;
    type Error = E;
}

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

impl<Fut> TryFlatten<Fut>
where
    Fut: TryFuture,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: TryFlattenInner::First { fut },
        }
    }
}

impl<Fut> Future for TryFlatten<Fut>
where
    Fut: TryFuture,
    Fut::Ok: TryFuture<Error = Fut::Error>,
{
    type Output = Result<<Fut::Ok as TryFuture>::Ok, Fut::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.project().inner;

        loop {
            match inner.as_mut().project() {
                TryFlattenInnerProject::First { fut } => {
                    let fut = futures::ready!(fut.poll(cx)?);

                    inner.set(TryFlattenInner::Second { fut });
                }
                TryFlattenInnerProject::Second { fut } => return fut.poll(cx),
            }
        }
    }
}

impl<Fut> FusedFuture for TryFlatten<Fut>
where
    Fut: TryFuture + FusedFuture,
    Fut::Ok: TryFuture<Error = Fut::Error> + FusedFuture,
{
    fn is_terminated(&self) -> bool {
        match &self.inner {
            TryFlattenInner::First { fut } => fut.is_terminated(),
            TryFlattenInner::Second { fut } => fut.is_terminated(),
        }
    }
}
