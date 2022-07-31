use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    #[project = FlattenInnerProject]
    enum FlattenInner<Fut>
    where
        Fut: Future,
    {
        First {
            #[pin]
            fut: Fut,
        },
        Second {
            #[pin]
            fut: Fut::Output,
        },
    }
}

pin_project_lite::pin_project! {
    pub struct Flatten<Fut>
    where
        Fut: Future,
    {
        #[pin]
        inner: FlattenInner<Fut>,
    }
}

impl<Fut> Flatten<Fut>
where
    Fut: Future,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: FlattenInner::First { fut },
        }
    }
}

impl<Fut> Future for Flatten<Fut>
where
    Fut: Future,
    Fut::Output: Future,
{
    type Output = <Fut::Output as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.project().inner;

        loop {
            match inner.as_mut().project() {
                FlattenInnerProject::First { fut } => {
                    let fut = futures_core::ready!(fut.poll(cx));

                    inner.set(FlattenInner::Second { fut });
                }
                FlattenInnerProject::Second { fut } => return fut.poll(cx),
            }
        }
    }
}

impl<Fut> FusedFuture for Flatten<Fut>
where
    Fut: FusedFuture,
    Fut::Output: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        match &self.inner {
            FlattenInner::First { fut } => fut.is_terminated(),
            FlattenInner::Second { fut } => fut.is_terminated(),
        }
    }
}
