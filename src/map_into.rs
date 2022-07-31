use crate::fn_mut_1::FnMut1;
use crate::map::Map;
use futures_core::FusedFuture;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

struct MapIntoFn<T, U> {
    _phantom: PhantomData<fn(T) -> U>,
}

impl<T, U> FnMut1<T> for MapIntoFn<T, U>
where
    T: Into<U>,
{
    type Output = U;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        arg.into()
    }
}

pin_project_lite::pin_project! {
    pub struct MapInto<Fut, U>
    where
        Fut: Future,
    {
        #[pin]
        inner: Map<Fut, MapIntoFn<Fut::Output, U>>,
    }
}

impl<Fut, U> MapInto<Fut, U>
where
    Fut: Future,
{
    pub(crate) fn new(fut: Fut) -> Self {
        Self {
            inner: Map::new(
                fut,
                MapIntoFn {
                    _phantom: PhantomData,
                },
            ),
        }
    }
}

impl<Fut, U> Future for MapInto<Fut, U>
where
    Fut: Future,
    Fut::Output: Into<U>,
{
    type Output = U;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<Fut, U> FusedFuture for MapInto<Fut, U>
where
    Fut: FusedFuture,
    Fut::Output: Into<U>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}
