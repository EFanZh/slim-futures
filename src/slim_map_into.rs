use futures::future::FusedFuture;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct SlimMapInto<T, U> {
        #[pin]
        fut: T,
        _phantom: PhantomData<fn(T) -> U>,
    }
}

impl<T, U> SlimMapInto<T, U> {
    pub(crate) fn new(fut: T) -> Self {
        Self {
            fut,
            _phantom: PhantomData,
        }
    }
}

impl<T, U> Future for SlimMapInto<T, U>
where
    T: Future,
    T::Output: Into<U>,
{
    type Output = U;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().fut.poll(cx).map(T::Output::into)
    }
}

impl<T, U> FusedFuture for SlimMapInto<T, U>
where
    T: FusedFuture,
    T::Output: Into<U>,
{
    fn is_terminated(&self) -> bool {
        self.fut.is_terminated()
    }
}
