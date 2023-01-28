use crate::future::Map;
use core::future::IntoFuture;
use core::marker::PhantomData;
use fn_traits::fns::ResultOkFn;
use fn_traits::FnMut;

pub struct MapOkAsyncFn<F, E>
where
    F: ?Sized,
{
    phantom: PhantomData<fn() -> E>,
    f: F,
}

impl<F, E> MapOkAsyncFn<F, E> {
    pub fn new(f: F) -> Self {
        Self {
            phantom: PhantomData,
            f,
        }
    }
}

impl<F, E> Clone for MapOkAsyncFn<F, E>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            phantom: self.phantom,
            f: self.f.clone(),
        }
    }
}

impl<T, F, E> FnMut<(T,)> for MapOkAsyncFn<F, E>
where
    F: FnMut<(T,)> + ?Sized,
    F::Output: IntoFuture,
{
    type Output = Map<<F::Output as IntoFuture>::IntoFuture, ResultOkFn<E>>;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        Map::new(self.f.call_mut(args).into_future(), ResultOkFn::default())
    }
}
