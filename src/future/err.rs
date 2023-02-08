use crate::future::map::Map;
use crate::future::ready::Ready;
use crate::support;
use core::future::Future;
use core::ops;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::{CloneFn, CopyFn, MemTakeFn, ResultErrFn};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct Err<G, T, E> {
        #[pin]
        inner: Map<Ready<G, E>, ResultErrFn<T>>
    }
}

impl<G, T, E> Err<G, T, E> {
    fn new(getter: G, error: E) -> Self {
        Self {
            inner: Map::new(Ready::new(getter, error), ResultErrFn::default()),
        }
    }
}

impl<G, T, E> Clone for Err<G, T, E>
where
    G: Clone,
    E: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<G, T, E> Future for Err<G, T, E>
where
    G: for<'a> FnMut<(&'a mut E,), Output = E>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

pub fn err_by<G, T, E>(getter: G, error: E) -> Err<G, T, E>
where
    G: ops::FnMut(&mut E) -> E,
{
    support::assert_future::<_, Result<T, E>>(Err::new(getter, error))
}

pub fn err_by_clone<T, E>(error: E) -> Err<CloneFn, T, E>
where
    E: Clone,
{
    support::assert_future::<_, Result<T, E>>(Err::new(CloneFn::default(), error))
}

pub fn err_by_copy<T, E>(error: E) -> Err<CopyFn, T, E>
where
    E: Copy,
{
    support::assert_future::<_, Result<T, E>>(Err::new(CopyFn::default(), error))
}

pub fn err_by_take<T, E>(error: E) -> Err<MemTakeFn, T, E>
where
    E: Default,
{
    support::assert_future::<_, Result<T, E>>(Err::new(MemTakeFn::default(), error))
}

#[cfg(test)]
mod tests {
    use crate::future::err;
    use std::mem;

    #[tokio::test]
    async fn test_err() {
        assert_eq!(err::err_by_copy::<u32, u32>(7).await, Err(7));
    }

    #[tokio::test]
    async fn test_err_clone() {
        let future = err::err_by_copy::<u32, u32>(7);
        let future_2 = future.clone();

        assert_eq!(future.await, Err(7));
        assert_eq!(future_2.await, Err(7));
    }

    #[tokio::test]
    async fn test_err_is_slim() {
        let value: u32 = 2;
        let future_1 = err::err_by_copy::<u32, _>(value);
        let future_2 = futures_util::future::err::<u32, _>(value);

        assert_eq!(mem::size_of_val(&value), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(future_1.await, Err(value));
        assert_eq!(future_2.await, Err(value));
    }
}
