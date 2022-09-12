# slim-futures

[![CI](https://github.com/EFanZh/slim-futures/actions/workflows/ci.yml/badge.svg)](https://github.com/EFanZh/slim-futures/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/EFanZh/slim-futures/branch/main/graph/badge.svg)](https://codecov.io/gh/EFanZh/slim-futures)
[![Coverage Status](https://coveralls.io/repos/github/EFanZh/slim-futures/badge.svg?branch=main)](https://coveralls.io/github/EFanZh/slim-futures?branch=main)

Async combinators that focus on small binary sizes.

| slim-futures                               | [`futures`]                      |
| ------------------------------------------ | -------------------------------- |
| `FutureExt::slim_and_then`                 |                                  |
| `FutureExt::slim_and_then_async`           | [`TryFutureExt::and_then`]       |
| `FutureExt::slim_err_into`                 | [`TryFutureExt::err_into`]       |
| `FutureExt::slim_flatten`                  | [`FutureExt::flatten`]           |
| `FutureExt::slim_flatten_async_iter`       | [`FutureExt::flatten_stream`]    |
| `FutureExt::slim_inspect`                  | [`FutureExt::inspect`]           |
| `FutureExt::slim_inspect_err`              | [`TryFutureExt::inspect_err`]    |
| `FutureExt::slim_inspect_ok`               | [`TryFutureExt::inspect_ok`]     |
| `FutureExt::slim_into_try_future`          |                                  |
| `FutureExt::slim_map`                      | [`FutureExt::map`]               |
| `FutureExt::slim_map_async`                | [`FutureExt::then`]              |
| `FutureExt::slim_map_err`                  | [`TryFutureExt::map_err`]        |
| `FutureExt::slim_map_err_async`            |                                  |
| `FutureExt::slim_map_into`                 | [`FutureExt::map_into`]          |
| `FutureExt::slim_map_ok`                   | [`TryFutureExt::map_ok`]         |
| `FutureExt::slim_map_ok_async`             |                                  |
| `FutureExt::slim_map_ok_or_else`           | [`TryFutureExt::map_ok_or_else`] |
| `FutureExt::slim_map_ok_or_else_async`     |                                  |
| `FutureExt::slim_never_error`              | [`FutureExt::never_error`]       |
| `FutureExt::slim_ok_into`                  | [`TryFutureExt::ok_into`]        |
| `FutureExt::slim_or_else`                  |                                  |
| `FutureExt::slim_or_else_async`            | [`TryFutureExt::or_else`]        |
| `FutureExt::slim_raw_map_ok_or_else_async` |                                  |
| `FutureExt::slim_try_flatten`              | [`TryFutureExt::try_flatten`]    |
| `FutureExt::slim_try_flatten_err`          |                                  |
| `FutureExt::slim_try_select_either`        |                                  |
| `FutureExt::slim_unit_error`               | [`FutureExt::unit_error`]        |
| `FutureExt::slim_unwrap_or_else`           | [`TryFutureExt::unwrap_or_else`] |
| `FutureExt::slim_unwrap_or_else_async`     |                                  |
| `err`                                      | [`err`]                          |
| `lazy`                                     | [`lazy`]                         |
| `ok`                                       | [`ok`]                           |
| `ready`                                    | [`ready`]                        |
| `raw_select`                               |                                  |
| `select_either`                            |                                  |

[`futures`]: https://docs.rs/futures/latest/futures/
[`FutureExt::flatten`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.flatten
[`FutureExt::flatten_stream`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.flatten_stream
[`FutureExt::inspect`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.inspect
[`FutureExt::map`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.map
[`FutureExt::map_into`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.map_into
[`FutureExt::never_error`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.never_error
[`FutureExt::then`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.then
[`FutureExt::unit_error`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.unit_error
[`TryFutureExt::and_then`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.and_then
[`TryFutureExt::err_into`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.err_into
[`TryFutureExt::inspect_err`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.inspect_err
[`TryFutureExt::inspect_ok`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.inspect_ok
[`TryFutureExt::map_err`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.map_err
[`TryFutureExt::map_ok`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.map_ok
[`TryFutureExt::map_ok_or_else`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.map_ok_or_else
[`TryFutureExt::ok_into`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.ok_into
[`TryFutureExt::or_else`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.or_else
[`TryFutureExt::try_flatten`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.try_flatten
[`TryFutureExt::unwrap_or_else`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.unwrap_or_else
[`err`]: https://docs.rs/futures/latest/futures/future/fn.err.html
[`lazy`]: https://docs.rs/futures/latest/futures/future/fn.lazy.html
[`ok`]: https://docs.rs/futures/latest/futures/future/fn.ok.html
[`ready`]: https://docs.rs/futures/latest/futures/future/fn.ready.html
