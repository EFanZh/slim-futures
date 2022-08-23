use crate::future::and_then::AndThen;
use crate::future::and_then_async::AndThenAsync;
use crate::future::flatten::Flatten;
use crate::future::inspect::Inspect;
use crate::future::map::Map;
use crate::future::map_async::MapAsync;
use crate::future::map_into::MapInto;
use crate::future::map_ok::MapOk;
use crate::future::map_ok_async::MapOkAsync;
use crate::future::select::Select;
use crate::future::try_flatten::TryFlatten;
use crate::support;
use std::future::Future;

pub trait FutureExt: Future {
    fn slim_and_then<F, T, E, U>(self, f: F) -> AndThen<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> Result<U, E>,
    {
        support::assert_future::<_, Result<U, E>>(AndThen::new(self, f))
    }

    fn slim_and_then_async<F, T, E, Fut2, U>(self, f: F) -> AndThenAsync<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> Fut2,
        Fut2: Future<Output = Result<U, E>>,
    {
        support::assert_future::<_, Result<U, E>>(AndThenAsync::new(self, f))
    }

    fn slim_flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Output: Future,
    {
        support::assert_future::<_, <Self::Output as Future>::Output>(Flatten::new(self))
    }

    fn slim_inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Output),
    {
        support::assert_future::<_, Self::Output>(Inspect::new(self, f))
    }

    fn slim_map<F, T>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> T,
    {
        support::assert_future::<_, T>(Map::new(self, f))
    }

    fn slim_map_async<F, Fut2>(self, f: F) -> MapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> Fut2,
        Fut2: Future,
    {
        support::assert_future::<_, Fut2::Output>(MapAsync::new(self, f))
    }

    fn slim_map_into<T>(self) -> MapInto<Self, T>
    where
        Self: Sized,
        Self::Output: Into<T>,
    {
        support::assert_future::<_, T>(MapInto::new(self))
    }

    fn slim_map_ok<F, T, E, U>(self, f: F) -> MapOk<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> U,
    {
        support::assert_future::<_, Result<U, E>>(MapOk::new(self, f))
    }

    fn slim_map_ok_async<F, T, E, Fut2>(self, f: F) -> MapOkAsync<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> Fut2,
        Fut2: Future,
    {
        support::assert_future::<_, Result<Fut2::Output, E>>(MapOkAsync::new(self, f))
    }

    fn slim_select<Fut>(self, fut: Fut) -> Select<Self, Fut>
    where
        Self: Sized,
        Fut: Future<Output = Self::Output>,
    {
        support::assert_future::<_, Self::Output>(Select::new(self, fut))
    }

    fn slim_try_flatten<Fut2, E, T>(self) -> TryFlatten<Self>
    where
        Self: Future<Output = Result<Fut2, E>> + Sized,
        Fut2: Future<Output = Result<T, E>> + Sized,
    {
        support::assert_future::<_, Result<T, E>>(TryFlatten::new(self))
    }
}

impl<Fut> FutureExt for Fut where Fut: Future + ?Sized {}
