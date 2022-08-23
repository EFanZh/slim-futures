use crate::future::map::Map;
use crate::support::FnMut1;
use futures_core::FusedFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct InspectFn<F> {
    inner: F,
}

impl<T, F> FnMut1<T> for InspectFn<F>
where
    F: for<'a> FnMut1<&'a T, Output = ()>,
{
    type Output = T;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        self.inner.call_mut(&arg);

        arg
    }
}

pin_project_lite::pin_project! {
    pub struct Inspect<Fut, F> {
        #[pin]
        inner: Map<Fut, InspectFn<F>>,
    }
}

impl<Fut, F> Inspect<Fut, F> {
    pub(crate) fn new(fut: Fut, f: F) -> Self {
        Self {
            inner: Map::new(fut, InspectFn { inner: f }),
        }
    }
}

impl<Fut, F> Future for Inspect<Fut, F>
where
    Fut: Future,
    F: FnMut(&Fut::Output),
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, F> FusedFuture for Inspect<Fut, F>
where
    Fut: FusedFuture,
    F: FnMut(&Fut::Output),
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}
