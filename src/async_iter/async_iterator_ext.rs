use crate::async_iter::all::All;
use crate::async_iter::all_async::AllAsync;
use crate::async_iter::and_then::AndThen;
use crate::async_iter::and_then_async::AndThenAsync;
use crate::async_iter::any::Any;
use crate::async_iter::any_async::AnyAsync;
use crate::async_iter::err_into::ErrInto;
use crate::async_iter::filter::Filter;
use crate::async_iter::filter_async::FilterAsync;
use crate::async_iter::filter_map::FilterMap;
use crate::async_iter::filter_map_async::FilterMapAsync;
use crate::async_iter::find::Find;
use crate::async_iter::find_async::FindAsync;
use crate::async_iter::find_map::FindMap;
use crate::async_iter::find_map_async::FindMapAsync;
use crate::async_iter::flat_map::FlatMap;
use crate::async_iter::flat_map_async::FlatMapAsync;
use crate::async_iter::flatten::Flatten;
use crate::async_iter::fold::Fold;
use crate::async_iter::fold_async::FoldAsync;
use crate::async_iter::for_each::ForEach;
use crate::async_iter::for_each_async::ForEachAsync;
use crate::async_iter::fuse::Fuse;
use crate::async_iter::inspect::Inspect;
use crate::async_iter::inspect_err::InspectErr;
use crate::async_iter::inspect_ok::InspectOk;
use crate::async_iter::map::Map;
use crate::async_iter::map_async::MapAsync;
use crate::async_iter::map_err::MapErr;
use crate::async_iter::map_err_async::MapErrAsync;
use crate::async_iter::map_ok::MapOk;
use crate::async_iter::map_ok_async::MapOkAsync;
use crate::async_iter::map_while::MapWhile;
use crate::async_iter::map_while_async::MapWhileAsync;
use crate::async_iter::ok_into::OkInto;
use crate::async_iter::or_else::OrElse;
use crate::async_iter::or_else_async::OrElseAsync;
use crate::async_iter::reduce::Reduce;
use crate::async_iter::reduce_async::ReduceAsync;
use crate::async_iter::scan::Scan;
use crate::async_iter::scan_async::ScanAsync;
use crate::async_iter::skip_while::SkipWhile;
use crate::async_iter::skip_while_async::SkipWhileAsync;
use crate::async_iter::take_while::TakeWhile;
use crate::async_iter::take_while_async::TakeWhileAsync;
use crate::async_iter::try_flatten::TryFlatten;
use crate::async_iter::try_fold::TryFold;
use crate::async_iter::try_fold_async::TryFoldAsync;
use crate::async_iter::try_for_each::TryForEach;
use crate::async_iter::try_for_each_async::TryForEachAsync;
use crate::async_iter::zip::Zip;
use crate::support::{AsyncIterator, FromResidual, IntoAsyncIterator, Residual, ResultAsyncIterator, Try};
use core::future::IntoFuture;
use fn_traits::fns::{CloneFn, CopyFn, MemTakeFn};

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

    fn slim_and_then<F, R>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized,
        Self::Item: Try,
        F: FnMut(<Self::Item as Try>::Output) -> R,
        R: FromResidual<<Self::Item as Try>::Residual>,
    {
        crate::support::assert_async_iter::<_, R>(AndThen::new(self, f))
    }

    fn slim_and_then_async<F, Fut>(self, f: F) -> AndThenAsync<Self, F>
    where
        Self: Sized,
        Self::Item: Try,
        F: FnMut(<Self::Item as Try>::Output) -> Fut,
        Fut: IntoFuture,
        Fut::Output: FromResidual<<Self::Item as Try>::Residual>,
    {
        crate::support::assert_async_iter::<_, Fut::Output>(AndThenAsync::new(self, f))
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

    fn slim_err_into<E>(self) -> ErrInto<Self, E>
    where
        Self: ResultAsyncIterator + Sized,
        Self::Error: Into<E>,
    {
        crate::support::assert_async_iter::<_, Result<Self::Ok, E>>(ErrInto::new(self))
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

    fn slim_find<P>(self, predicate: P) -> Find<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        crate::support::assert_future::<_, Option<Self::Item>>(Find::new(self, predicate))
    }

    fn slim_find_async<P, Fut>(self, predicate: P) -> FindAsync<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> Fut,
        Fut: IntoFuture<Output = bool>,
    {
        crate::support::assert_future::<_, Option<Self::Item>>(FindAsync::new(self, predicate))
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

    fn slim_fold_by<T, G, F>(self, init: T, getter: G, f: F) -> Fold<Self, T, G, F>
    where
        Self: Sized,
        G: FnMut(&mut T) -> T,
        F: FnMut(T, Self::Item) -> T,
    {
        crate::support::assert_future::<_, T>(Fold::new(self, init, getter, f))
    }

    fn slim_fold_by_copy<T, F>(self, init: T, f: F) -> Fold<Self, T, CopyFn, F>
    where
        Self: Sized,
        T: Copy,
        F: FnMut(T, Self::Item) -> T,
    {
        crate::support::assert_future::<_, T>(Fold::new(self, init, CopyFn::default(), f))
    }

    fn slim_fold_by_clone<T, F>(self, init: T, f: F) -> Fold<Self, T, CloneFn, F>
    where
        Self: Sized,
        T: Clone,
        F: FnMut(T, Self::Item) -> T,
    {
        crate::support::assert_future::<_, T>(Fold::new(self, init, CloneFn::default(), f))
    }

    fn slim_fold_by_take<T, F>(self, init: T, f: F) -> Fold<Self, T, MemTakeFn, F>
    where
        Self: Sized,
        T: Default,
        F: FnMut(T, Self::Item) -> T,
    {
        crate::support::assert_future::<_, T>(Fold::new(self, init, MemTakeFn::default(), f))
    }

    fn slim_fold_async_by<T, G, F, Fut>(self, init: T, getter: G, f: F) -> FoldAsync<Self, T, G, F>
    where
        Self: Sized,
        G: FnMut(&mut T) -> T,
        F: FnMut(T, Self::Item) -> Fut,
        Fut: IntoFuture<Output = T>,
    {
        crate::support::assert_future::<_, T>(FoldAsync::new(self, init, getter, f))
    }

    fn slim_fold_async_by_copy<T, F, Fut>(self, init: T, f: F) -> FoldAsync<Self, T, CopyFn, F>
    where
        Self: Sized,
        T: Copy,
        F: FnMut(T, Self::Item) -> Fut,
        Fut: IntoFuture<Output = T>,
    {
        crate::support::assert_future::<_, T>(FoldAsync::new(self, init, CopyFn::default(), f))
    }

    fn slim_fold_async_by_clone<T, F, Fut>(self, init: T, f: F) -> FoldAsync<Self, T, CloneFn, F>
    where
        Self: Sized,
        T: Clone,
        F: FnMut(T, Self::Item) -> Fut,
        Fut: IntoFuture<Output = T>,
    {
        crate::support::assert_future::<_, T>(FoldAsync::new(self, init, CloneFn::default(), f))
    }

    fn slim_fold_async_by_take<T, F, Fut>(self, init: T, f: F) -> FoldAsync<Self, T, MemTakeFn, F>
    where
        Self: Sized,
        T: Default,
        F: FnMut(T, Self::Item) -> Fut,
        Fut: IntoFuture<Output = T>,
    {
        crate::support::assert_future::<_, T>(FoldAsync::new(self, init, MemTakeFn::default(), f))
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

    fn slim_fuse(self) -> Fuse<Self>
    where
        Self: Sized,
    {
        crate::support::assert_async_iter::<_, Self::Item>(Fuse::new(self))
    }

    fn slim_inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item),
    {
        crate::support::assert_async_iter::<_, Self::Item>(Inspect::new(self, f))
    }

    fn slim_inspect_err<F>(self, f: F) -> InspectErr<Self, F>
    where
        Self: ResultAsyncIterator + Sized,
        F: FnMut(&Self::Error),
    {
        crate::support::assert_async_iter::<_, Self::Item>(InspectErr::new(self, f))
    }

    fn slim_inspect_ok<F>(self, f: F) -> InspectOk<Self, F>
    where
        Self: ResultAsyncIterator + Sized,
        F: FnMut(&Self::Ok),
    {
        crate::support::assert_async_iter::<_, Self::Item>(InspectOk::new(self, f))
    }

    fn slim_map<F, T>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> T,
    {
        crate::support::assert_async_iter::<_, T>(Map::new(self, f))
    }

    fn slim_map_async<F, Fut>(self, f: F) -> MapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: IntoFuture,
    {
        crate::support::assert_async_iter::<_, Fut::Output>(MapAsync::new(self, f))
    }

    fn slim_map_err<F, E>(self, f: F) -> MapErr<Self, F>
    where
        Self: ResultAsyncIterator + Sized,
        F: FnMut(Self::Error) -> E,
    {
        crate::support::assert_async_iter::<_, Result<Self::Ok, E>>(MapErr::new(self, f))
    }

    fn slim_map_err_async<F, Fut>(self, f: F) -> MapErrAsync<Self, F>
    where
        Self: ResultAsyncIterator + Sized,
        F: FnMut(Self::Error) -> Fut,
        Fut: IntoFuture,
    {
        crate::support::assert_async_iter::<_, Result<Self::Ok, Fut::Output>>(MapErrAsync::new(self, f))
    }

    fn slim_map_ok<F, T>(self, f: F) -> MapOk<Self, F>
    where
        Self: Sized,
        Self::Item: Try,
        <Self::Item as Try>::Residual: Residual<T>,
        F: FnMut(<Self::Item as Try>::Output) -> T,
    {
        crate::support::assert_async_iter::<_, <<Self::Item as Try>::Residual as Residual<T>>::TryType>(MapOk::new(
            self, f,
        ))
    }

    fn slim_map_ok_async<F, Fut>(self, f: F) -> MapOkAsync<Self, F>
    where
        Self: Sized,
        Self::Item: Try,
        <Self::Item as Try>::Residual: Residual<Fut::Output>,
        F: FnMut(<Self::Item as Try>::Output) -> Fut,
        Fut: IntoFuture,
    {
        crate::support::assert_async_iter::<_, <<Self::Item as Try>::Residual as Residual<Fut::Output>>::TryType>(
            MapOkAsync::new(self, f),
        )
    }

    fn slim_map_while<F, T>(self, f: F) -> MapWhile<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<T>,
    {
        crate::support::assert_async_iter::<_, T>(MapWhile::new(self, f))
    }

    fn slim_map_while_async<F, Fut, T>(self, f: F) -> MapWhileAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: IntoFuture<Output = Option<T>>,
    {
        crate::support::assert_async_iter::<_, T>(MapWhileAsync::new(self, f))
    }

    fn slim_ok_into<T>(self) -> OkInto<Self, T>
    where
        Self: AsyncIterator + Sized,
        Self::Item: Try,
        <Self::Item as Try>::Output: Into<T>,
        <Self::Item as Try>::Residual: Residual<T>,
    {
        crate::support::assert_async_iter::<_, <<Self::Item as Try>::Residual as Residual<T>>::TryType>(OkInto::new(
            self,
        ))
    }

    fn slim_or_else<F, R>(self, f: F) -> OrElse<Self, F>
    where
        Self: ResultAsyncIterator + Sized,
        F: FnMut(Self::Error) -> R,
        R: Try<Output = Self::Ok>,
    {
        crate::support::assert_async_iter::<_, R>(OrElse::new(self, f))
    }

    fn slim_or_else_async<F, Fut>(self, f: F) -> OrElseAsync<Self, F>
    where
        Self: ResultAsyncIterator + Sized,
        F: FnMut(Self::Error) -> Fut,
        Fut: IntoFuture,
        Fut::Output: Try<Output = Self::Ok>,
    {
        crate::support::assert_async_iter::<_, Fut::Output>(OrElseAsync::new(self, f))
    }

    fn slim_reduce<F>(self, f: F) -> Reduce<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item, Self::Item) -> Self::Item,
    {
        crate::support::assert_future::<_, Option<Self::Item>>(Reduce::new(self, f))
    }

    fn slim_reduce_async<F, Fut>(self, f: F) -> ReduceAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item, Self::Item) -> Fut,
        Fut: IntoFuture<Output = Self::Item>,
    {
        crate::support::assert_future::<_, Option<Self::Item>>(ReduceAsync::new(self, f))
    }

    fn slim_scan<S, F, T>(self, state: S, f: F) -> Scan<Self, S, F>
    where
        Self: Sized,
        F: FnMut(&mut S, Self::Item) -> Option<T>,
    {
        crate::support::assert_async_iter::<_, T>(Scan::new(self, state, f))
    }

    fn slim_scan_async<S, F, Fut, T>(self, state: S, f: F) -> ScanAsync<Self, S, F>
    where
        Self: Sized,
        F: FnMut(&mut S, Self::Item) -> Fut,
        Fut: IntoFuture<Output = Option<T>>,
    {
        crate::support::assert_async_iter::<_, T>(ScanAsync::new(self, state, f))
    }

    fn slim_skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        crate::support::assert_async_iter::<_, Self::Item>(SkipWhile::new(self, predicate))
    }

    fn slim_skip_while_async<P, Fut>(self, predicate: P) -> SkipWhileAsync<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> Fut,
        Fut: IntoFuture<Output = bool>,
    {
        crate::support::assert_async_iter::<_, Self::Item>(SkipWhileAsync::new(self, predicate))
    }

    fn slim_take_while<P>(self, predicate: P) -> TakeWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        crate::support::assert_async_iter::<_, Self::Item>(TakeWhile::new(self, predicate))
    }

    fn slim_take_while_async<P, Fut>(self, predicate: P) -> TakeWhileAsync<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> Fut,
        Fut: IntoFuture<Output = bool>,
    {
        crate::support::assert_async_iter::<_, Self::Item>(TakeWhileAsync::new(self, predicate))
    }

    fn slim_try_flatten(self) -> TryFlatten<Self>
    where
        Self: Sized,
        Self::Item: Try,
        <Self::Item as Try>::Output: IntoAsyncIterator,
        <<Self::Item as Try>::Output as IntoAsyncIterator>::Item: FromResidual<<Self::Item as Try>::Residual>,
    {
        crate::support::assert_async_iter::<_, <<Self::Item as Try>::Output as IntoAsyncIterator>::Item>(
            TryFlatten::new(self),
        )
    }

    fn slim_try_fold_by<T, G, F, R>(self, init: T, getter: G, f: F) -> TryFold<Self, T, G, F>
    where
        Self: Sized,
        G: FnMut(&mut T) -> T,
        F: FnMut(T, Self::Item) -> R,
        R: Try<Output = T>,
    {
        crate::support::assert_future::<_, R>(TryFold::new(self, init, getter, f))
    }

    fn slim_try_fold_by_copy<T, F, R>(self, init: T, f: F) -> TryFold<Self, T, CopyFn, F>
    where
        Self: Sized,
        T: Copy,
        F: FnMut(T, Self::Item) -> R,
        R: Try<Output = T>,
    {
        crate::support::assert_future::<_, R>(TryFold::new(self, init, CopyFn::default(), f))
    }

    fn slim_try_fold_by_clone<T, F, R>(self, init: T, f: F) -> TryFold<Self, T, CloneFn, F>
    where
        Self: Sized,
        T: Clone,
        F: FnMut(T, Self::Item) -> R,
        R: Try<Output = T>,
    {
        crate::support::assert_future::<_, R>(TryFold::new(self, init, CloneFn::default(), f))
    }

    fn slim_try_fold_by_take<T, F, R>(self, init: T, f: F) -> TryFold<Self, T, MemTakeFn, F>
    where
        Self: Sized,
        T: Default,
        F: FnMut(T, Self::Item) -> R,
        R: Try<Output = T>,
    {
        crate::support::assert_future::<_, R>(TryFold::new(self, init, MemTakeFn::default(), f))
    }

    fn slim_try_fold_async_by<T, G, F, Fut>(self, init: T, getter: G, f: F) -> TryFoldAsync<Self, T, G, F>
    where
        Self: Sized,
        G: FnMut(&mut T) -> T,
        F: FnMut(T, Self::Item) -> Fut,
        Fut: IntoFuture,
        Fut::Output: Try<Output = T>,
    {
        crate::support::assert_future::<_, Fut::Output>(TryFoldAsync::new(self, init, getter, f))
    }

    fn slim_try_fold_async_by_copy<T, F, Fut>(self, init: T, f: F) -> TryFoldAsync<Self, T, CopyFn, F>
    where
        Self: Sized,
        T: Copy,
        F: FnMut(T, Self::Item) -> Fut,
        Fut: IntoFuture,
        Fut::Output: Try<Output = T>,
    {
        crate::support::assert_future::<_, Fut::Output>(TryFoldAsync::new(self, init, CopyFn::default(), f))
    }

    fn slim_try_fold_async_by_clone<T, F, Fut>(self, init: T, f: F) -> TryFoldAsync<Self, T, CloneFn, F>
    where
        Self: Sized,
        T: Clone,
        F: FnMut(T, Self::Item) -> Fut,
        Fut: IntoFuture,
        Fut::Output: Try<Output = T>,
    {
        crate::support::assert_future::<_, Fut::Output>(TryFoldAsync::new(self, init, CloneFn::default(), f))
    }

    fn slim_try_fold_async_by_take<T, F, Fut>(self, init: T, f: F) -> TryFoldAsync<Self, T, MemTakeFn, F>
    where
        Self: Sized,
        T: Default,
        F: FnMut(T, Self::Item) -> Fut,
        Fut: IntoFuture,
        Fut::Output: Try<Output = T>,
    {
        crate::support::assert_future::<_, Fut::Output>(TryFoldAsync::new(self, init, MemTakeFn::default(), f))
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

    fn slim_zip<I>(self, other: I) -> Zip<Self, I::IntoAsyncIter>
    where
        Self: Sized,
        I: IntoAsyncIterator,
    {
        crate::support::assert_async_iter::<_, (Self::Item, I::Item)>(Zip::new(self, other.into_async_iter()))
    }
}

impl<I> AsyncIteratorExt for I where I: AsyncIterator {}
