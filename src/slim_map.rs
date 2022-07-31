use crate::fn_mut_1::FnMut1;
use futures::future::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct SlimMap<Fut, F> {
        #[pin]
        fut: Fut,
        f: F,
    }
}

impl<Fut, F> SlimMap<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self { fut, f }
    }
}

impl<Fut, F> Future for SlimMap<Fut, F>
where
    Fut: Future,
    F: FnMut1<Fut::Output>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        this.fut.poll(cx).map(|value| this.f.call_mut(value))
    }
}

impl<Fut, F> FusedFuture for SlimMap<Fut, F>
where
    Fut: FusedFuture,
    F: FnMut1<Fut::Output>,
{
    fn is_terminated(&self) -> bool {
        self.fut.is_terminated()
    }
}
