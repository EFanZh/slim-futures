use crate::flatten::Flatten;
use crate::fn_mut_1::FnMut1;
use crate::map::Map;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct MapAsync<Fut, F>
    where
        Fut: Future,
        F: FnMut1<Fut::Output>,
    {
        #[pin]
        inner: Flatten<Map<Fut, F>>
    }
}

impl<Fut, F> MapAsync<Fut, F>
where
    Fut: Future,
    F: FnMut1<Fut::Output>,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Flatten::new(Map::new(fut, f)),
        }
    }
}

impl<Fut, F> Future for MapAsync<Fut, F>
where
    Fut: Future,
    F: FnMut1<Fut::Output>,
    F::Output: Future,
{
    type Output = <F::Output as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for MapAsync<Fut, F>
where
    Fut: FusedFuture,
    F: FnMut1<Fut::Output>,
    F::Output: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}
