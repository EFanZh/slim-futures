use crate::future::map::Map;
use crate::future::ready::Ready;
use crate::support;
use core::future::Future;
use core::ops;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::{CloneFn, CopyFn, MemTakeFn, ResultOkFn};
use fn_traits::FnMut;

pin_project_lite::pin_project! {
    pub struct Ok<G, T, E> {
        #[pin]
        inner: Map<Ready<G, T>, ResultOkFn<E>>
    }
}

impl<G, T, E> Ok<G, T, E> {
    fn new(getter: G, value: T) -> Self {
        Self {
            inner: Map::new(Ready::new(getter, value), ResultOkFn::default()),
        }
    }
}

impl<G, T, E> Clone for Ok<G, T, E>
where
    G: Clone,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<G, T, E> Future for Ok<G, T, E>
where
    G: for<'a> FnMut<(&'a mut T,), Output = T>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

pub fn ok_by<G, T, E>(getter: G, value: T) -> Ok<G, T, E>
where
    G: ops::FnMut(&mut T) -> T,
{
    support::assert_future::<_, Result<T, E>>(Ok::new(getter, value))
}

pub fn ok_by_copy<T, E>(value: T) -> Ok<CopyFn, T, E>
where
    T: Copy,
{
    support::assert_future::<_, Result<T, E>>(Ok::new(CopyFn::default(), value))
}

pub fn ok_by_clone<T, E>(value: T) -> Ok<CloneFn, T, E>
where
    T: Clone,
{
    support::assert_future::<_, Result<T, E>>(Ok::new(CloneFn::default(), value))
}

pub fn ok_by_take<T, E>(value: T) -> Ok<MemTakeFn, T, E>
where
    T: Default,
{
    support::assert_future::<_, Result<T, E>>(Ok::new(MemTakeFn::default(), value))
}

#[cfg(test)]
mod tests {
    use crate::future;
    use std::mem;

    #[tokio::test]
    async fn test_ok() {
        assert_eq!(future::ok_by_copy::<u32, u32>(2).await, Ok(2));
    }

    #[tokio::test]
    async fn test_ok_clone() {
        let future = future::ok_by_copy::<u32, u32>(2);
        let future_2 = future.clone();

        assert_eq!(future.await, Ok(2));
        assert_eq!(future_2.await, Ok(2));
    }

    #[tokio::test]
    async fn test_ok_is_slim() {
        let value: u32 = 2;
        let future_1 = future::ok_by_copy::<_, u32>(value);
        let future_2 = futures_util::future::ok::<_, u32>(value);

        assert_eq!(mem::size_of_val(&value), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(future_1.await, Ok(value));
        assert_eq!(future_2.await, Ok(value));
    }
}
