use crate::async_iter::all::All;
use crate::async_iter::all_async::AllAsync;
use crate::async_iter::any::Any;
use crate::async_iter::any_async::AnyAsync;
use crate::async_iter::filter::Filter;
use crate::async_iter::filter_async::FilterAsync;
use crate::async_iter::filter_map::FilterMap;
use crate::async_iter::filter_map_async::FilterMapAsync;
use crate::async_iter::find_map::FindMap;
use crate::async_iter::find_map_async::FindMapAsync;
use crate::async_iter::flatten::Flatten;
use crate::async_iter::fold::Fold;
use crate::async_iter::fold_async::FoldAsync;
use crate::async_iter::try_fold::TryFold;
use crate::async_iter::try_fold_async::TryFoldAsync;
use crate::support::{AsyncIterator, IntoAsyncIterator, Try};
use core::future::Future;

pub trait AsyncIteratorExt: AsyncIterator {
    fn slim_all<P>(self, predicate: P) -> All<Self, P>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        crate::support::assert_future::<_, bool>(All::new(self, predicate))
    }

    fn slim_all_async<P, Fut>(self, predicate: P) -> AllAsync<Self, P>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> Fut,
        Fut: Future<Output = bool>,
    {
        crate::support::assert_future::<_, bool>(AllAsync::new(self, predicate))
    }

    fn slim_any<P>(self, predicate: P) -> Any<Self, P>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        crate::support::assert_future::<_, bool>(Any::new(self, predicate))
    }

    fn slim_any_async<P, Fut>(self, predicate: P) -> AnyAsync<Self, P>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> Fut,
        Fut: Future<Output = bool>,
    {
        crate::support::assert_future::<_, bool>(AnyAsync::new(self, predicate))
    }

    fn slim_filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        crate::support::assert_async_iter::<_, Self::Item>(Filter::new(self, predicate))
    }

    fn slim_filter_async<P, Fut>(self, predicate: P) -> FilterAsync<Self, P, Fut>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> Fut,
        Fut: Future<Output = bool>,
    {
        crate::support::assert_async_iter::<_, Self::Item>(FilterAsync::new(self, predicate))
    }

    fn slim_filter_map<F, B>(self, f: F) -> FilterMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        crate::support::assert_async_iter::<_, B>(FilterMap::new(self, f))
    }

    fn slim_filter_map_async<F, Fut, B>(self, f: F) -> FilterMapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: Future<Output = Option<B>>,
    {
        crate::support::assert_async_iter::<_, B>(FilterMapAsync::new(self, f))
    }

    fn slim_find_map<F, B>(self, f: F) -> FindMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        crate::support::assert_future::<_, Option<B>>(FindMap::new(self, f))
    }

    fn slim_find_map_async<F, Fut, B>(self, f: F) -> FindMapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: Future<Output = Option<B>>,
    {
        crate::support::assert_future::<_, Option<B>>(FindMapAsync::new(self, f))
    }

    fn slim_flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Item: IntoAsyncIterator,
    {
        crate::support::assert_async_iter::<_, <Self::Item as IntoAsyncIterator>::Item>(Flatten::new(self))
    }

    fn slim_fold<B, F>(self, init: B, f: F) -> Fold<Self, B, F>
    where
        Self: Sized,
        B: Copy,
        F: FnMut(B, Self::Item) -> B,
    {
        crate::support::assert_future::<_, B>(Fold::new(self, init, f))
    }

    fn slim_fold_async<B, F, Fut>(self, init: B, f: F) -> FoldAsync<Self, B, F>
    where
        Self: Sized,
        B: Copy,
        F: FnMut(B, Self::Item) -> Fut,
        Fut: Future<Output = B>,
    {
        crate::support::assert_future::<_, B>(FoldAsync::new(self, init, f))
    }

    fn slim_try_fold<B, F, R>(self, init: B, f: F) -> TryFold<Self, B, F>
    where
        Self: Sized,
        B: Copy,
        F: FnMut(B, Self::Item) -> R,
        R: Try<Output = B>,
    {
        crate::support::assert_future::<_, R>(TryFold::new(self, init, f))
    }

    fn slim_try_fold_async<B, F, Fut>(self, init: B, f: F) -> TryFoldAsync<Self, B, F>
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
