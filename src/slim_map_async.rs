use crate::fn_mut_1::FnMut1;
use crate::slim_flatten::SlimFlatten;
use crate::slim_map::SlimMap;
use futures::future::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project_lite::pin_project! {
    pub struct SlimMapAsync<Fut, F>
    where
        Fut: Future,
        F: FnMut1<Fut::Output>,
    {
        #[pin]
        inner: SlimFlatten<SlimMap<Fut, F>>
    }
}

impl<Fut, F> SlimMapAsync<Fut, F>
where
    Fut: Future,
    F: FnMut1<Fut::Output>,
{
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: SlimFlatten::new(SlimMap::new(fut, f)),
        }
    }
}

impl<Fut, F> Future for SlimMapAsync<Fut, F>
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

impl<Fut, F> FusedFuture for SlimMapAsync<Fut, F>
where
    Fut: FusedFuture,
    F: FnMut1<Fut::Output>,
    F::Output: FusedFuture,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}
