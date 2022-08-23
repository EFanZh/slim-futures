pub use self::fn_mut_1::FnMut1;
pub use self::try_fn_mut_1::TryFnMut1;
pub use self::try_future::TryFuture;

mod fn_mut_1;
mod try_fn_mut_1;
mod try_future;

pub fn assert_future<Fut, T>(fut: Fut) -> Fut {
    fut
}
