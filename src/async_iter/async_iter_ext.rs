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
use crate::async_iter::flat_map::FlatMap;
use crate::async_iter::flat_map_async::FlatMapAsync;
use crate::async_iter::flatten::Flatten;
use crate::async_iter::fold::Fold;
use crate::async_iter::fold_async::FoldAsync;
use crate::async_iter::for_each::ForEach;
use crate::async_iter::for_each_async::ForEachAsync;
use crate::async_iter::inspect::Inspect;
use crate::async_iter::map::Map;
use crate::async_iter::map_async::MapAsync;
use crate::async_iter::map_while::MapWhile;
use crate::async_iter::try_fold::TryFold;
use crate::async_iter::try_fold_async::TryFoldAsync;
use crate::async_iter::try_for_each::TryForEach;
use crate::async_iter::try_for_each_async::TryForEachAsync;
use crate::support::{AsyncIterator, IntoAsyncIterator, Try};
use core::future::IntoFuture;

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
        Fut: IntoFuture<Output = bool>,
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
        Fut: IntoFuture<Output = bool>,
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

    fn slim_filter_async<P, Fut>(self, predicate: P) -> FilterAsync<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> Fut,
        Fut: IntoFuture<Output = bool>,
    {
        crate::support::assert_async_iter::<_, Self::Item>(FilterAsync::new(self, predicate))
    }

    fn slim_filter_map<F, T>(self, f: F) -> FilterMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<T>,
    {
        crate::support::assert_async_iter::<_, T>(FilterMap::new(self, f))
    }

    fn slim_filter_map_async<F, Fut, T>(self, f: F) -> FilterMapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: IntoFuture<Output = Option<T>>,
    {
        crate::support::assert_async_iter::<_, T>(FilterMapAsync::new(self, f))
    }

    fn slim_find_map<F, T>(self, f: F) -> FindMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<T>,
    {
        crate::support::assert_future::<_, Option<T>>(FindMap::new(self, f))
    }

    fn slim_find_map_async<F, Fut, T>(self, f: F) -> FindMapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: IntoFuture<Output = Option<T>>,
    {
        crate::support::assert_future::<_, Option<T>>(FindMapAsync::new(self, f))
    }

    fn slim_flat_map<F, I>(self, f: F) -> FlatMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> I,
        I: IntoAsyncIterator,
    {
        crate::support::assert_async_iter::<_, I::Item>(FlatMap::new(self, f))
    }

    fn slim_flat_map_async<F, Fut>(self, f: F) -> FlatMapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: IntoFuture,
        Fut::Output: IntoAsyncIterator,
    {
        crate::support::assert_async_iter::<_, <<F::Output as IntoFuture>::Output as IntoAsyncIterator>::Item>(
            FlatMapAsync::new(self, f),
        )
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
        Fut: IntoFuture<Output = B>,
    {
        crate::support::assert_future::<_, B>(FoldAsync::new(self, init, f))
    }

    fn slim_for_each<F>(self, f: F) -> ForEach<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        crate::support::assert_future::<_, ()>(ForEach::new(self, f))
    }

    fn slim_for_each_async<F, Fut>(self, f: F) -> ForEachAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: IntoFuture<Output = ()>,
    {
        crate::support::assert_future::<_, ()>(ForEachAsync::new(self, f))
    }

    fn slim_inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item),
    {
        crate::support::assert_async_iter::<_, Self::Item>(Inspect::new(self, f))
    }

    fn slim_map<F, B>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B,
    {
        crate::support::assert_async_iter::<_, B>(Map::new(self, f))
    }

    fn slim_map_async<F, Fut>(self, f: F) -> MapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: IntoFuture,
    {
        crate::support::assert_async_iter::<_, Fut::Output>(MapAsync::new(self, f))
    }

    fn slim_map_while<F, B>(self, f: F) -> MapWhile<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        crate::support::assert_async_iter::<_, B>(MapWhile::new(self, f))
    }

    fn slim_try_fold<T, F, R>(self, init: T, f: F) -> TryFold<Self, T, F>
    where
        Self: Sized,
        T: Copy,
        F: FnMut(T, Self::Item) -> R,
        R: Try<Output = T>,
    {
        crate::support::assert_future::<_, R>(TryFold::new(self, init, f))
    }

    fn slim_try_fold_async<T, F, Fut>(self, init: T, f: F) -> TryFoldAsync<Self, T, F>
    where
        Self: Sized,
        T: Copy,
        F: FnMut(T, Self::Item) -> Fut,
        Fut: IntoFuture,
        Fut::Output: Try<Output = T>,
    {
        crate::support::assert_future::<_, Fut::Output>(TryFoldAsync::new(self, init, f))
    }

    fn slim_try_for_each<F, R>(self, f: F) -> TryForEach<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>,
    {
        crate::support::assert_future::<_, R>(TryForEach::new(self, f))
    }

    fn slim_try_for_each_async<F, Fut>(self, f: F) -> TryForEachAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: IntoFuture,
        Fut::Output: Try<Output = ()>,
    {
        crate::support::assert_future::<_, Fut::Output>(TryForEachAsync::new(self, f))
    }
}

impl<I> AsyncIteratorExt for I where I: AsyncIterator {}
