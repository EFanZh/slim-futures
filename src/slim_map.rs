use futures::future::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct SlimMap<T, F> {
        #[pin]
        fut: T,
        f: F,
    }
}

impl<T, F> SlimMap<T, F> {
    pub(crate) fn new(fut: T, f: F) -> Self {
        Self { fut, f }
    }
}

impl<T, F, R> Future for SlimMap<T, F>
where
    T: Future,
    F: FnMut(T::Output) -> R,
{
    type Output = R;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        this.fut.poll(cx).map(this.f)
    }
}

impl<T, F, R> FusedFuture for SlimMap<T, F>
where
    T: FusedFuture,
    F: FnMut(T::Output) -> R,
{
    fn is_terminated(&self) -> bool {
        self.fut.is_terminated()
    }
}
