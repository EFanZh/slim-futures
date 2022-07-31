use futures::future::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct Select<Fut1, Fut2> {
        #[pin]
        fut_1: Fut1,
        #[pin]
        fut_2: Fut2,
    }
}

impl<Fut1, Fut2> Select<Fut1, Fut2> {
    pub(crate) fn new(fut_1: Fut1, fut_2: Fut2) -> Self {
        Self { fut_1, fut_2 }
    }
}

impl<Fut1, Fut2> Future for Select<Fut1, Fut2>
where
    Fut1: Future,
    Fut2: Future<Output = Fut1::Output>,
{
    type Output = Fut1::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        match this.fut_1.poll(cx) {
            Poll::Ready(result) => Poll::Ready(result),
            Poll::Pending => this.fut_2.poll(cx),
        }
    }
}

impl<Fut1, Fut2> FusedFuture for Select<Fut1, Fut2>
where
    Fut1: FusedFuture,
    Fut2: FusedFuture<Output = Fut1::Output>,
{
    fn is_terminated(&self) -> bool {
        self.fut_1.is_terminated() || self.fut_2.is_terminated()
    }
}
