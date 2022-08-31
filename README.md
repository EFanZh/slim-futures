# slim-futures

[![CI](https://github.com/EFanZh/slim-futures/actions/workflows/ci.yml/badge.svg)](https://github.com/EFanZh/slim-futures/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/EFanZh/slim-futures/branch/main/graph/badge.svg)](https://codecov.io/gh/EFanZh/slim-futures)
[![Coverage Status](https://coveralls.io/repos/github/EFanZh/slim-futures/badge.svg?branch=main)](https://coveralls.io/github/EFanZh/slim-futures?branch=main)

Async combinators that focus on small binary sizes.

| slim-futures                             | [`futures`]                   |
| ---------------------------------------- | ----------------------------- |
| `SlimFutureExt::slim_and_then`           |                               |
| `SlimFutureExt::slim_and_then_async`     | [`TryFutureExt::and_then`]    |
| `SlimFutureExt::slim_err_into`           | [`TryFutureExt::err_into`]    |
| `SlimFutureExt::slim_flatten`            | [`FutureExt::flatten`]        |
| `SlimFutureExt::slim_flatten_async_iter` | [`FutureExt::flatten_stream`] |
| `SlimFutureExt::slim_inspect`            | [`FutureExt::inspect`]        |
| `SlimFutureExt::slim_inspect_err`        | [`TryFutureExt::inspect_err`] |
| `SlimFutureExt::slim_inspect_ok`         | [`TryFutureExt::inspect_ok`]  |
| `SlimFutureExt::slim_into_try_future`    |                               |
| `SlimFutureExt::slim_map`                | [`FutureExt::map`]            |
| `SlimFutureExt::slim_map_async`          | [`FutureExt::then`]           |
| `SlimFutureExt::slim_map_err`            | [`TryFutureExt::map_err`]     |
| `SlimFutureExt::slim_map_into`           | [`FutureExt::map_into`]       |
| `SlimFutureExt::slim_never_error`        | [`FutureExt::never_error`]    |
| `SlimFutureExt::slim_or_else_async`      | [`TryFutureExt::or_else`]     |
| `SlimFutureExt::slim_unit_error`         | [`FutureExt::unit_error`]     |

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
[`TryFutureExt::or_else`]: https://docs.rs/futures/latest/futures/future/trait.TryFutureExt.html#method.or_else
