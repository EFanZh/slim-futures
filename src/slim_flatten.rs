use futures::future::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    #[project = SlimFlattenInnerProject]
    enum SlimFlattenInner<Fut>
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
    pub struct SlimFlatten<Fut>
    where
        Fut: Future,
    {
        #[pin]
        inner: SlimFlattenInner<Fut>,
    }
}

impl<Fut> SlimFlatten<Fut>
where
    Fut: Future,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: SlimFlattenInner::First { fut },
        }
    }
}

impl<Fut> Future for SlimFlatten<Fut>
where
    Fut: Future,
    Fut::Output: Future,
{
    type Output = <Fut::Output as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.project().inner;

        loop {
            match inner.as_mut().project() {
                SlimFlattenInnerProject::First { fut } => {
                    let fut = futures::ready!(fut.poll(cx));

                    inner.set(SlimFlattenInner::Second { fut });
                }
                SlimFlattenInnerProject::Second { fut } => return fut.poll(cx),
            }
        }
    }
}

impl<Fut> FusedFuture for SlimFlatten<Fut>
where
    Fut: FusedFuture,
    Fut::Output: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        match &self.inner {
            SlimFlattenInner::First { fut } => fut.is_terminated(),
            SlimFlattenInner::Second { fut } => fut.is_terminated(),
        }
    }
}
