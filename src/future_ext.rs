use crate::and_then::AndThen;
use crate::and_then_async::AndThenAsync;
use crate::assert_future;
use crate::flatten::Flatten;
use crate::inspect::Inspect;
use crate::map::Map;
use crate::map_async::MapAsync;
use crate::map_into::MapInto;
use crate::map_ok::MapOk;
use crate::select::Select;
use crate::try_flatten::TryFlatten;
use std::future::Future;

pub trait FutureExt: Future {
    fn slim_and_then<F, T, E, U>(self, f: F) -> AndThen<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> Result<U, E>,
    {
        assert_future::assert_future::<_, Result<U, E>>(AndThen::new(self, f))
    }

    fn slim_and_then_async<F, T, E, U, V>(self, f: F) -> AndThenAsync<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> U,
        U: Future<Output = Result<V, E>>,
    {
        assert_future::assert_future::<_, Result<V, E>>(AndThenAsync::new(self, f))
    }

    fn slim_flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Output: Future,
    {
        assert_future::assert_future::<_, <Self::Output as Future>::Output>(Flatten::new(self))
    }

    fn slim_inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Output),
    {
        assert_future::assert_future::<_, Self::Output>(Inspect::new(self, f))
    }

    fn slim_map<F, U>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> U,
    {
        assert_future::assert_future::<_, U>(Map::new(self, f))
    }

    fn slim_map_async<F, U>(self, f: F) -> MapAsync<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> U,
        U: Future,
    {
        assert_future::assert_future::<_, U::Output>(MapAsync::new(self, f))
    }

    fn slim_map_into<U>(self) -> MapInto<Self, U>
    where
        Self: Sized,
        Self::Output: Into<U>,
    {
        assert_future::assert_future::<_, U>(MapInto::new(self))
    }

    fn slim_map_ok<F, T, E, U>(self, f: F) -> MapOk<Self, F>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        F: FnMut(T) -> U,
    {
        assert_future::assert_future::<_, Result<U, E>>(MapOk::new(self, f))
    }

    fn slim_select<Fut>(self, fut: Fut) -> Select<Self, Fut>
    where
        Self: Sized,
        Fut: Future<Output = Self::Output>,
    {
        assert_future::assert_future::<_, Self::Output>(Select::new(self, fut))
    }

    fn slim_try_flatten<T, E, U>(self) -> TryFlatten<Self>
    where
        Self: Future<Output = Result<T, E>> + Sized,
        T: Future<Output = Result<U, E>> + Sized,
    {
        assert_future::assert_future::<_, Result<U, E>>(TryFlatten::new(self))
    }
}

impl<T> FutureExt for T where T: Future + ?Sized {}
