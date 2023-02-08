use crate::support;
use core::future::Future;
use core::ops;
use core::pin::Pin;
use core::task::{Context, Poll};
use fn_traits::FnMut;

#[derive(Clone)]
pub struct Lazy<F>
where
    F: ?Sized,
{
    f: F,
}

impl<F> Unpin for Lazy<F> where F: ?Sized {}

impl<F, T> Future for Lazy<F>
where
    F: for<'a, 'b> FnMut<(&'a mut Context<'b>,), Output = T> + ?Sized,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(self.f.call_mut((cx,)))
    }
}

pub fn lazy<F, T>(f: F) -> Lazy<F>
where
    F: ops::FnMut(&mut Context) -> T,
{
    support::assert_future::<_, T>(Lazy { f })
}

#[cfg(test)]
mod tests {
    use std::mem;
    use std::task::Context;

    fn lazy_fn(_: &mut Context) -> u32 {
        2
    }

    #[tokio::test]
    async fn test_lazy() {
        assert_eq!(super::lazy(lazy_fn).await, 2);
    }

    #[tokio::test]
    async fn test_lazy_clone() {
        let future = super::lazy(lazy_fn);
        let future_2 = future.clone();

        assert_eq!(future.await, 2);
        assert_eq!(future_2.await, 2);
    }

    #[tokio::test]
    async fn test_lazy_is_slim() {
        let future_1 = super::lazy(lazy_fn);
        let future_2 = futures_util::future::lazy(lazy_fn);

        assert_eq!(mem::size_of_val(&lazy_fn), mem::size_of_val(&future_1));
        assert!(mem::size_of_val(&future_1) < mem::size_of_val(&future_2));
        assert_eq!(future_1.await, 2);
        assert_eq!(future_2.await, 2);
    }
}
