use crate::assert_future;
use crate::async_slim_map::AsyncSlimMap;
use crate::slim_flatten::SlimFlatten;
use crate::slim_inspect::SlimInspect;
use crate::slim_map::SlimMap;
use crate::slim_map_into::SlimMapInto;
use std::future::Future;

pub trait SlimFutureExt: Future {
    fn async_slim_map<F, U>(self, f: F) -> AsyncSlimMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> U,
        U: Future,
    {
        assert_future::assert_future::<_, U::Output>(AsyncSlimMap::new(self, f))
    }

    fn slim_flatten(self) -> SlimFlatten<Self>
    where
        Self: Sized,
        Self::Output: Future,
    {
        assert_future::assert_future::<_, <Self::Output as Future>::Output>(SlimFlatten::new(self))
    }

    fn slim_inspect<F>(self, f: F) -> SlimInspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Output),
    {
        assert_future::assert_future::<_, Self::Output>(SlimInspect::new(self, f))
    }

    fn slim_map_into<U>(self) -> SlimMapInto<Self, U>
    where
        Self: Sized,
        Self::Output: Into<U>,
    {
        assert_future::assert_future::<_, U>(SlimMapInto::new(self))
    }

    fn slim_map<F, U>(self, f: F) -> SlimMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> U,
    {
        assert_future::assert_future::<_, U>(SlimMap::new(self, f))
    }
}

impl<T> SlimFutureExt for T where T: Future + ?Sized {}
