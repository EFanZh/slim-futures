use futures::future::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct SlimInspect<T, F> {
        #[pin]
        fut: T,
        f: F,
    }
}

impl<T, F> SlimInspect<T, F> {
    pub(crate) fn new(fut: T, f: F) -> Self {
        Self { fut, f }
    }
}

impl<T, F> Future for SlimInspect<T, F>
where
    T: Future,
    F: FnMut(&T::Output),
{
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        this.fut.poll(cx).map(|value| {
            (this.f)(&value);

            value
        })
    }
}

impl<T, F> FusedFuture for SlimInspect<T, F>
where
    T: FusedFuture,
    F: FnMut(&T::Output),
{
    fn is_terminated(&self) -> bool {
        self.fut.is_terminated()
    }
}
