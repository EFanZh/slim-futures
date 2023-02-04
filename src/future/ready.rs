use crate::support;
use core::future::Future;
use core::ops;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::fns::{CloneFn, CopyFn, MemTakeFn};
use fn_traits::FnMut;

#[derive(Clone)]
pub struct Ready<G, T> {
    getter: G,
    value: T,
}

impl<G, T> Ready<G, T> {
    pub(crate) fn new(getter: G, value: T) -> Self {
        Self { getter, value }
    }
}

impl<G, T> Unpin for Ready<G, T> {}

impl<G, T> Future for Ready<G, T>
where
    G: for<'a> FnMut<(&'a mut T,), Output = T>,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
        let this = &mut *self;

        Poll::Ready(this.getter.call_mut((&mut this.value,)))
    }
}

pub fn ready_by<G, T>(getter: G, value: T) -> Ready<G, T>
where
    G: ops::FnMut(&mut T) -> T,
{
    support::assert_future::<_, T>(Ready::new(getter, value))
}

pub fn ready_by_clone<T>(value: T) -> Ready<CloneFn, T>
where
    T: Clone,
{
    support::assert_future::<_, T>(Ready::new(CloneFn::default(), value))
}

pub fn ready_by_copy<T>(value: T) -> Ready<CopyFn, T>
where
    T: Copy,
{
    support::assert_future::<_, T>(Ready::new(CopyFn::default(), value))
}

pub fn ready_by_take<T>(value: T) -> Ready<MemTakeFn, T>
where
    T: Default,
{
    support::assert_future::<_, T>(Ready::new(MemTakeFn::default(), value))
}

#[cfg(test)]
mod tests {
    use crate::future;
    use std::mem;

    #[tokio::test]
    async fn test_ready() {
        assert_eq!(future::ready_by_copy::<u32>(2).await, 2);
    }

    #[tokio::test]
    async fn test_ready_clone() {
        let future = future::ready_by_copy::<u32>(2);
        let future_2 = future.clone();

        assert_eq!(future.await, 2);
        assert_eq!(future_2.await, 2);
    }

    #[tokio::test]
    async fn test_ready_is_slim() {
        let value: u32 = 2;
        let future_1 = future::ready_by_copy(value);
        let future_2 = futures_util::future::ready(value);

        assert_eq!(mem::size_of_val(&value), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(future_1.await, 2);
        assert_eq!(future_2.await, 2);
    }
}
