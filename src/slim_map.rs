use crate::fn_mut_1::FnMut1;
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

impl<T, F> Future for SlimMap<T, F>
where
    T: Future,
    F: FnMut1<T::Output>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        this.fut.poll(cx).map(this.f.as_fn_mut())
    }
}
