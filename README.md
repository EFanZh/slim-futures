# slim-futures

[![CI](https://github.com/EFanZh/slim-futures/actions/workflows/ci.yml/badge.svg)](https://github.com/EFanZh/slim-futures/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/EFanZh/slim-futures/branch/main/graph/badge.svg)](https://codecov.io/gh/EFanZh/slim-futures)
[![Coverage Status](https://coveralls.io/repos/github/EFanZh/slim-futures/badge.svg?branch=main)](https://coveralls.io/github/EFanZh/slim-futures?branch=main)

Provides [`Future`] and [`Stream`] combinators as a supplement for [`futures`] crate, but focus on small memory
occupation and small binary size.

Advantages:

- Have minimal memory occupation.
- Some combinators have less internal states than their [`futures`] counterparts, which could be beneficial for binary
  size.
- Supports more [`Try`] types, rather than only [`Result`] type.
- Provides both asynchronous and synchronous version of combinators, where the asynchronous ones have a `_async` suffix.
- Provides extra combinators that is not implemented by [`futures`] crate.

Disadvantages:

- Combinators do not support [`FnOnce`] functions.
- Unless it’s free, combinators do not implement [`FusedFuture`] and [`FusedStream`].
- Some combinators requires user to provide a method for taking ownership of a value by mutable reference. For example,
  `AsyncIteratorExt::slim_fold_by*` combinators requires user to specify a method for taking the value out of the
  future, either by [`Copy`]ing, [`Clone::clone`], [`mem::take`], or some other user provided function.
- Currently, the project is in its early stage, APIs are not stabilized, and the implementations have not been fully
  verified, use with caution.

Here is the list of combinators provided by `slim-futures`, and their [`futures`] counterparts.

| [`slim-futures`]                            | [`futures`]                      | Notes                               |
| ------------------------------------------- | -------------------------------- | ----------------------------------- |
| `AsyncIteratorExt::slim_all`                |                                  |                                     |
| `AsyncIteratorExt::slim_all_async`          | [`StreamExt::all`]               |                                     |
| `AsyncIteratorExt::slim_and_then`           |                                  |                                     |
| `AsyncIteratorExt::slim_and_then_async`     | [`TryStreamExt::and_then`]       |                                     |
| `AsyncIteratorExt::slim_any`                |                                  |                                     |
| `AsyncIteratorExt::slim_any_async`          | [`StreamExt::any`]               |                                     |
| `AsyncIteratorExt::slim_filter`             |                                  |                                     |
| `AsyncIteratorExt::slim_filter_async`       | [`StreamExt::filter`]            |                                     |
| `AsyncIteratorExt::slim_filter_map`         |                                  |                                     |
| `AsyncIteratorExt::slim_filter_map_async`   | [`StreamExt::filter_map`]        |                                     |
| `AsyncIteratorExt::slim_find`               |                                  |                                     |
| `AsyncIteratorExt::slim_find_async`         |                                  |                                     |
| `AsyncIteratorExt::slim_find_map`           |                                  |                                     |
| `AsyncIteratorExt::slim_find_map_async`     |                                  |                                     |
| `AsyncIteratorExt::slim_flat_map`           |                                  |                                     |
| `AsyncIteratorExt::slim_flat_map_async`     | [`StreamExt::flat_map`]          |                                     |
| `AsyncIteratorExt::slim_flatten`            | [`StreamExt::flatten`]           |                                     |
| `AsyncIteratorExt::slim_fold_by*`           |                                  |                                     |
| `AsyncIteratorExt::slim_fold_async_by*`     | [`StreamExt::fold`]              |                                     |
| `AsyncIteratorExt::slim_for_each`           |                                  |                                     |
| `AsyncIteratorExt::slim_for_each_async`     | [`StreamExt::for_each`]          |                                     |
| `AsyncIteratorExt::slim_fuse`               | [`StreamExt::fuse`]              |                                     |
| `AsyncIteratorExt::slim_inspect`            | [`StreamExt::inspect`]           |                                     |
| `AsyncIteratorExt::slim_map`                | [`StreamExt::map`]               |                                     |
| `AsyncIteratorExt::slim_map_async`          | [`StreamExt::then`]              |                                     |
| `AsyncIteratorExt::slim_map_err`            | [`TryStreamExt::map_err`]        |                                     |
| `AsyncIteratorExt::slim_map_err_async`      |                                  |                                     |
| `AsyncIteratorExt::slim_map_ok`             | [`TryStreamExt::map_ok`]         |                                     |
| `AsyncIteratorExt::slim_map_ok_async`       |                                  |                                     |
| `AsyncIteratorExt::slim_map_while`          |                                  |                                     |
| `AsyncIteratorExt::slim_map_while_async`    |                                  |                                     |
| `AsyncIteratorExt::slim_or_else`            |                                  |                                     |
| `AsyncIteratorExt::slim_or_else_async`      | [`TryStreamExt::or_else`]        |                                     |
| `AsyncIteratorExt::slim_reduce`             |                                  |                                     |
| `AsyncIteratorExt::slim_reduce_async`       |                                  |                                     |
| `AsyncIteratorExt::slim_scan`               |                                  |                                     |
| `AsyncIteratorExt::slim_scan_async`         | [`StreamExt::scan`]              |                                     |
| `AsyncIteratorExt::slim_skip_while`         |                                  |                                     |
| `AsyncIteratorExt::slim_skip_while_async`   | [`StreamExt::skip_while`]        |                                     |
| `AsyncIteratorExt::slim_take_while`         |                                  |                                     |
| `AsyncIteratorExt::slim_take_while_async`   | [`StreamExt::take_while`]        |                                     |
| `AsyncIteratorExt::slim_try_fold_by*`       |                                  |                                     |
| `AsyncIteratorExt::slim_try_fold_async_by*` | [`TryStreamExt::try_fold`]       | Follows [`Iterator::try_fold`].     |
| `AsyncIteratorExt::slim_try_for_each`       |                                  |                                     |
| `AsyncIteratorExt::slim_try_for_each_async` | [`TryStreamExt::try_for_each`]   | Follows [`Iterator::try_for_each`]. |
| `AsyncIteratorExt::slim_zip`                | [`StreamExt::zip`]               |                                     |
| `FutureExt::slim_and_then`                  |                                  |                                     |
| `FutureExt::slim_and_then_async`            | [`TryFutureExt::and_then`]       |                                     |
| `FutureExt::slim_err_into`                  | [`TryFutureExt::err_into`]       |                                     |
| `FutureExt::slim_flatten`                   | [`FutureExt::flatten`]           |                                     |
| `FutureExt::slim_flatten_async_iter`        | [`FutureExt::flatten_stream`]    |                                     |
| `FutureExt::slim_inspect`                   | [`FutureExt::inspect`]           |                                     |
| `FutureExt::slim_inspect_err`               | [`TryFutureExt::inspect_err`]    |                                     |
| `FutureExt::slim_inspect_ok`                | [`TryFutureExt::inspect_ok`]     |                                     |
| `FutureExt::slim_into_result_future`        |                                  |                                     |
| `FutureExt::slim_map`                       | [`FutureExt::map`]               |                                     |
| `FutureExt::slim_map_async`                 | [`FutureExt::then`]              |                                     |
| `FutureExt::slim_map_err`                   | [`TryFutureExt::map_err`]        |                                     |
| `FutureExt::slim_map_err_async`             |                                  |                                     |
| `FutureExt::slim_map_into`                  | [`FutureExt::map_into`]          |                                     |
| `FutureExt::slim_map_ok`                    | [`TryFutureExt::map_ok`]         |                                     |
| `FutureExt::slim_map_ok_async`              |                                  |                                     |
| `FutureExt::slim_map_ok_or_else`            | [`TryFutureExt::map_ok_or_else`] |                                     |
| `FutureExt::slim_map_ok_or_else_async`      |                                  |                                     |
| `FutureExt::slim_never_error`               | [`FutureExt::never_error`]       |                                     |
| `FutureExt::slim_ok_into`                   | [`TryFutureExt::ok_into`]        |                                     |
| `FutureExt::slim_or_else`                   |                                  |                                     |
| `FutureExt::slim_or_else_async`             | [`TryFutureExt::or_else`]        |                                     |
| `FutureExt::slim_raw_map_ok_or_else_async`  |                                  |                                     |
| `FutureExt::slim_try_flatten`               | [`TryFutureExt::try_flatten`]    |                                     |
| `FutureExt::slim_try_flatten_err`           |                                  |                                     |
| `FutureExt::slim_unit_error`                | [`FutureExt::unit_error`]        |                                     |
| `FutureExt::slim_unwrap_or_else`            | [`TryFutureExt::unwrap_or_else`] |                                     |
| `FutureExt::slim_unwrap_or_else_async`      |                                  |                                     |
| `err_by*`                                   | [`err`]                          |                                     |
| `lazy`                                      | [`lazy`]                         |                                     |
| `ok_by*`                                    | [`ok`]                           |                                     |
| `raw_select`                                |                                  |                                     |
| `ready_by*`                                 | [`ready`]                        |                                     |
| `select_either`                             |                                  |                                     |
| `try_select_either`                         |                                  |                                     |

[`Clone::clone`]: https://doc.rust-lang.org/stable/std/clone/trait.Clone.html#tymethod.clone
[`Copy`]: https://doc.rust-lang.org/stable/std/marker/trait.Copy.html
[`FnOnce`]: https://doc.rust-lang.org/stable/std/ops/trait.FnOnce.html
[`Future`]: https://doc.rust-lang.org/stable/std/future/trait.Future.html
[`Result`]: https://doc.rust-lang.org/stable/std/result/enum.Result.html
[`Try`]: https://doc.rust-lang.org/stable/std/ops/trait.Try.html
[`FusedFuture`]: https://docs.rs/futures/latest/futures/future/trait.FusedFuture.html
[`FusedStream`]: https://docs.rs/futures/latest/futures/stream/trait.FusedStream.html
[`Stream`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
[`mem::take`]: https://doc.rust-lang.org/stable/std/mem/fn.take.html
[`futures`]: https://docs.rs/futures/latest/futures/
[`slim-futures`]: https://github.com/EFanZh/slim-futures
[`FutureExt::flatten`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.flatten
[`FutureExt::flatten_stream`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.flatten_stream
[`FutureExt::inspect`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.inspect
[`FutureExt::map`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.map
[`FutureExt::map_into`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.map_into
[`FutureExt::never_error`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.never_error
[`FutureExt::then`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.then
[`FutureExt::unit_error`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.unit_error
[`StreamExt::all`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.all
[`StreamExt::any`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.any
[`StreamExt::filter`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.filter
[`StreamExt::filter_map`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.filter_map
[`StreamExt::flat_map`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.flat_map
[`StreamExt::flatten`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.flatten
[`StreamExt::fold`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.fold
[`StreamExt::for_each`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.for_each
[`StreamExt::fuse`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.fuse
[`StreamExt::inspect`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.inspect
[`StreamExt::map`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.map
[`StreamExt::scan`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.scan
[`StreamExt::skip_while`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.skip_while
[`StreamExt::take_while`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.take_while
[`StreamExt::then`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.then
[`StreamExt::zip`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.zip
[`Iterator::try_fold`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.try_fold
[`Iterator::try_for_each`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.try_for_each
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
[`TryStreamExt::and_then`]: https://docs.rs/futures/latest/futures/stream/trait.TryStreamExt.html#method.and_then
[`TryStreamExt::map_err`]: https://docs.rs/futures/latest/futures/stream/trait.TryStreamExt.html#method.map_err
[`TryStreamExt::map_ok`]: https://docs.rs/futures/latest/futures/stream/trait.TryStreamExt.html#method.map_ok
[`TryStreamExt::or_else`]: https://docs.rs/futures/latest/futures/stream/trait.TryStreamExt.html#method.or_else
[`TryStreamExt::try_fold`]: https://docs.rs/futures/latest/futures/stream/trait.TryStreamExt.html#method.try_fold
[`TryStreamExt::try_for_each`]: https://docs.rs/futures/latest/futures/stream/trait.TryStreamExt.html#method.try_for_each
[`err`]: https://docs.rs/futures/latest/futures/future/fn.err.html
[`lazy`]: https://docs.rs/futures/latest/futures/future/fn.lazy.html
[`ok`]: https://docs.rs/futures/latest/futures/future/fn.ok.html
[`ready`]: https://docs.rs/futures/latest/futures/future/fn.ready.html
