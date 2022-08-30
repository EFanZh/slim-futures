#![allow(clippy::module_name_repetitions)] // False positive, for `IntoTryFuture`,

pub use self::and_then::AndThen;
pub use self::and_then_async::AndThenAsync;
pub use self::err_into::ErrInto;
pub use self::flatten::Flatten;
pub use self::future_ext::FutureExt as SlimFutureExt;
pub use self::inspect::Inspect;
pub use self::inspect_err::InspectErr;
pub use self::inspect_ok::InspectOk;
pub use self::into_try_future::IntoTryFuture;
pub use self::lazy::{lazy, Lazy};
pub use self::map::Map;
pub use self::map_async::MapAsync;
pub use self::map_err::MapErr;
pub use self::map_into::MapInto;
pub use self::map_ok::MapOk;
pub use self::map_ok_async::MapOkAsync;
pub use self::or_else_async::OrElseAsync;
pub use self::ready::{ready, Ready};
pub use self::select::Select;
pub use self::try_flatten::TryFlatten;
pub use self::try_flatten_err::TryFlattenErr;

mod and_then;
mod and_then_async;
mod err_into;
mod flatten;
mod future_ext;
mod inspect;
mod inspect_err;
mod inspect_ok;
mod into_try_future;
mod lazy;
mod map;
mod map_async;
mod map_err;
mod map_into;
mod map_ok;
mod map_ok_async;
mod or_else_async;
mod ready;
mod select;
mod try_flatten;
mod try_flatten_err;
