pub use self::into_async_iterator::IntoAsyncIterator;
pub use self::into_result_future::IntoResultFuture;
pub use self::option_future::OptionFuture;
pub use self::predicate_fn::PredicateFn;
pub use self::raw_residual::RawResidual;
pub use self::result_async_iterator::ResultAsyncIterator;
pub use self::result_future::ResultFuture;
pub use self::try_::{FromResidual, Try};
pub use core::convert::Infallible as Never;
use futures_core::Future;
pub use futures_core::{FusedStream as FusedAsyncIterator, Stream as AsyncIterator};

pub mod fns;
mod into_async_iterator;
mod into_result_future;
mod option_future;
mod predicate_fn;
mod raw_residual;
mod result_async_iterator;
mod result_future;
pub mod states;
mod try_;

pub fn assert_future<Fut, T>(fut: Fut) -> Fut
where
    Fut: Future<Output = T>,
{
    fut
}

pub fn assert_async_iter<I, T>(iter: I) -> I
where
    I: AsyncIterator<Item = T>,
{
    iter
}
