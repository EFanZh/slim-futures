use crate::async_iter::all::All;
use crate::async_iter::all_async::AllAsync;
use crate::async_iter::any::Any;
use crate::async_iter::any_async::AnyAsync;
use crate::async_iter::filter::Filter;
use crate::async_iter::fold::Fold;
use crate::async_iter::fold_async::FoldAsync;
use crate::async_iter::try_fold::TryFold;
use crate::async_iter::try_fold_async::TryFoldAsync;
use crate::support::{AsyncIterator, Try};
use core::future::Future;

pub trait AsyncIteratorExt: AsyncIterator {
    fn all<F>(self, f: F) -> All<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        crate::support::assert_future::<_, bool>(All::new(self, f))
    }

    fn all_async<F, Fut>(self, f: F) -> AllAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: Future<Output = bool>,
    {
        crate::support::assert_future::<_, bool>(AllAsync::new(self, f))
    }

    fn any<F>(self, f: F) -> Any<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        crate::support::assert_future::<_, bool>(Any::new(self, f))
    }

    fn any_async<F, Fut>(self, f: F) -> AnyAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: Future<Output = bool>,
    {
        crate::support::assert_future::<_, bool>(AnyAsync::new(self, f))
    }

    fn fold<B, F>(self, init: B, f: F) -> Fold<Self, B, F>
    where
        Self: Sized,
        B: Copy,
        F: FnMut(B, Self::Item) -> B,
    {
        crate::support::assert_future::<_, B>(Fold::new(self, init, f))
    }

    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        crate::support::assert_async_iter::<_, Self::Item>(Filter::new(self, predicate))
    }

    fn fold_async<B, F, Fut>(self, init: B, f: F) -> FoldAsync<Self, B, F>
    where
        Self: Sized,
        B: Copy,
        F: FnMut(B, Self::Item) -> Fut,
        Fut: Future<Output = B>,
    {
        crate::support::assert_future::<_, B>(FoldAsync::new(self, init, f))
    }

    fn try_fold<B, F, R>(self, init: B, f: F) -> TryFold<Self, B, F>
    where
        Self: Sized,
        B: Copy,
        F: FnMut(B, Self::Item) -> R,
        R: Try<Output = B>,
    {
        crate::support::assert_future::<_, R>(TryFold::new(self, init, f))
    }

    fn try_fold_async<B, F, Fut>(self, init: B, f: F) -> TryFoldAsync<Self, B, F>
    where
        Self: Sized,
        B: Copy,
        F: FnMut(B, Self::Item) -> Fut,
        Fut: Future,
        Fut::Output: Try<Output = B>,
    {
        crate::support::assert_future::<_, Fut::Output>(TryFoldAsync::new(self, init, f))
    }
}

impl<I> AsyncIteratorExt for I where I: AsyncIterator {}
