use futures::future::FusedFuture;
use futures::TryFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    #[project = SlimTryFlattenInnerProject]
    enum SlimTryFlattenInner<T>
    where
        T: TryFuture,
    {
        First {
            #[pin]
            fut: T,
        },
        Second {
            #[pin]
            fut: T::Ok,
        },
    }
}

pin_project_lite::pin_project! {
    pub struct SlimTryFlatten<T>
    where
        T: TryFuture,
    {
        #[pin]
        inner: SlimTryFlattenInner<T>,
    }
}

impl<T> SlimTryFlatten<T>
where
    T: TryFuture,
{
    pub(crate) fn new(fut: T) -> Self {
        Self {
            inner: SlimTryFlattenInner::First { fut },
        }
    }
}

impl<T> Future for SlimTryFlatten<T>
where
    T: TryFuture,
    T::Ok: TryFuture<Error = T::Error>,
{
    type Output = Result<<T::Ok as TryFuture>::Ok, T::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.project().inner;

        loop {
            match inner.as_mut().project() {
                SlimTryFlattenInnerProject::First { fut } => {
                    let fut = futures::ready!(fut.try_poll(cx)?);

                    inner.set(SlimTryFlattenInner::Second { fut });
                }
                SlimTryFlattenInnerProject::Second { fut } => return fut.try_poll(cx),
            }
        }
    }
}

impl<T> FusedFuture for SlimTryFlatten<T>
where
    T: TryFuture + FusedFuture,
    T::Ok: TryFuture<Error = T::Error> + FusedFuture,
{
    fn is_terminated(&self) -> bool {
        match &self.inner {
            SlimTryFlattenInner::First { fut } => fut.is_terminated(),
            SlimTryFlattenInner::Second { fut } => fut.is_terminated(),
        }
    }
}
