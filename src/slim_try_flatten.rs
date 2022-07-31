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
    #[project = SlimTryFlattenInnerProject]
    enum SlimTryFlattenInner<Fut>
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
    pub struct SlimTryFlatten<Fut>
    where
        Fut: TryFuture,
    {
        #[pin]
        inner: SlimTryFlattenInner<Fut>,
    }
}

impl<Fut> SlimTryFlatten<Fut>
where
    Fut: TryFuture,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: SlimTryFlattenInner::First { fut },
        }
    }
}

impl<Fut> Future for SlimTryFlatten<Fut>
where
    Fut: TryFuture,
    Fut::Ok: TryFuture<Error = Fut::Error>,
{
    type Output = Result<<Fut::Ok as TryFuture>::Ok, Fut::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.project().inner;

        loop {
            match inner.as_mut().project() {
                SlimTryFlattenInnerProject::First { fut } => {
                    let fut = futures::ready!(fut.poll(cx)?);

                    inner.set(SlimTryFlattenInner::Second { fut });
                }
                SlimTryFlattenInnerProject::Second { fut } => return fut.poll(cx),
            }
        }
    }
}

impl<Fut> FusedFuture for SlimTryFlatten<Fut>
where
    Fut: TryFuture + FusedFuture,
    Fut::Ok: TryFuture<Error = Fut::Error> + FusedFuture,
{
    fn is_terminated(&self) -> bool {
        match &self.inner {
            SlimTryFlattenInner::First { fut } => fut.is_terminated(),
            SlimTryFlattenInner::Second { fut } => fut.is_terminated(),
        }
    }
}
