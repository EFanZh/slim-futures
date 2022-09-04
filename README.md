# slim-futures

[![CI](https://github.com/EFanZh/slim-futures/actions/workflows/ci.yml/badge.svg)](https://github.com/EFanZh/slim-futures/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/EFanZh/slim-futures/branch/main/graph/badge.svg)](https://codecov.io/gh/EFanZh/slim-futures)
[![Coverage Status](https://coveralls.io/repos/github/EFanZh/slim-futures/badge.svg?branch=main)](https://coveralls.io/github/EFanZh/slim-futures?branch=main)

Async combinators that focus on small binary sizes.

| slim-futures                                       | [`futures`]                              |
| -------------------------------------------------- | ---------------------------------------- |
| `future::SlimFutureExt::slim_and_then`             |                                          |
| `future::SlimFutureExt::slim_and_then_async`       | [`future::TryFutureExt::and_then`]       |
| `future::SlimFutureExt::slim_err_into`             | [`future::TryFutureExt::err_into`]       |
| `future::SlimFutureExt::slim_flatten`              | [`future::FutureExt::flatten`]           |
| `future::SlimFutureExt::slim_flatten_async_iter`   | [`future::FutureExt::flatten_stream`]    |
| `future::SlimFutureExt::slim_inspect`              | [`future::FutureExt::inspect`]           |
| `future::SlimFutureExt::slim_inspect_err`          | [`future::TryFutureExt::inspect_err`]    |
| `future::SlimFutureExt::slim_inspect_ok`           | [`future::TryFutureExt::inspect_ok`]     |
| `future::SlimFutureExt::slim_into_try_future`      |                                          |
| `future::SlimFutureExt::slim_map`                  | [`future::FutureExt::map`]               |
| `future::SlimFutureExt::slim_map_async`            | [`future::FutureExt::then`]              |
| `future::SlimFutureExt::slim_map_err`              | [`future::TryFutureExt::map_err`]        |
| `future::SlimFutureExt::slim_map_err_async`        |                                          |
| `future::SlimFutureExt::slim_map_into`             | [`future::FutureExt::map_into`]          |
| `future::SlimFutureExt::slim_map_ok`               | [`future::TryFutureExt::map_ok`]         |
| `future::SlimFutureExt::slim_map_ok_async`         |                                          |
| `future::SlimFutureExt::slim_never_error`          | [`future::FutureExt::never_error`]       |
| `future::SlimFutureExt::slim_ok_into`              | [`future::TryFutureExt::ok_into`]        |
| `future::SlimFutureExt::slim_or_else`              |                                          |
| `future::SlimFutureExt::slim_or_else_async`        | [`future::TryFutureExt::or_else`]        |
| `future::SlimFutureExt::slim_try_flatten`          | [`future::TryFutureExt::try_flatten`]    |
| `future::SlimFutureExt::slim_try_flatten_err`      |                                          |
| `future::SlimFutureExt::slim_unit_error`           | [`future::FutureExt::unit_error`]        |
| `future::SlimFutureExt::slim_unwrap_or_else`       | [`future::TryFutureExt::unwrap_or_else`] |
| `future::SlimFutureExt::slim_unwrap_or_else_async` |                                          |
| `future::err`                                      | [`future::err`]                          |

[`futures`]: https://docs.rs/futures/latest/futures/
[`future::FutureExt::flatten`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.flatten
[`future::FutureExt::flatten_stream`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.flatten_stream
[`future::FutureExt::inspect`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.inspect
[`future::FutureExt::map`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.map
[`future::FutureExt::map_into`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.map_into
[`future::FutureExt::never_error`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.never_error
[`future::FutureExt::then`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.then
[`future::FutureExt::unit_error`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.unit_error
[`future::TryFutureExt::and_then`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.and_then
[`future::TryFutureExt::err_into`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.err_into
[`future::TryFutureExt::inspect_err`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.inspect_err
[`future::TryFutureExt::inspect_ok`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.inspect_ok
[`future::TryFutureExt::map_err`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.map_err
[`future::TryFutureExt::map_ok`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.map_ok
[`future::TryFutureExt::ok_into`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.ok_into
[`future::TryFutureExt::or_else`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.or_else
[`future::TryFutureExt::try_flatten`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.try_flatten
[`future::TryFutureExt::unwrap_or_else`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.unwrap_or_else
[`future::err`]: https://docs.rs/futures/latest/futures/future/fn.err.html
