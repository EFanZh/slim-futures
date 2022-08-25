# slim-futures

[![CI](https://github.com/EFanZh/slim-futures/actions/workflows/ci.yml/badge.svg)](https://github.com/EFanZh/slim-futures/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/EFanZh/slim-futures/branch/main/graph/badge.svg)](https://codecov.io/gh/EFanZh/slim-futures)
[![Coverage Status](https://coveralls.io/repos/github/EFanZh/slim-futures/badge.svg?branch=main)](https://coveralls.io/github/EFanZh/slim-futures?branch=main)

Async combinators that focus on small binary sizes.

| slim-futures                    | [`futures`]             |
| ------------------------------- | ----------------------- |
| `SlimFutureExt::slim_flatten`   | [`FutureExt::flatten`]  |
| `SlimFutureExt::slim_inspect`   | [`FutureExt::inspect`]  |
| `SlimFutureExt::slim_map`       | [`FutureExt::map`]      |
| `SlimFutureExt::slim_map_async` | [`FutureExt::then`]     |
| `SlimFutureExt::slim_map_into`  | [`FutureExt::map_into`] |

[`futures`]: https://docs.rs/futures/latest/futures/
[`FutureExt::flatten`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.flatten
[`FutureExt::inspect`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.inspect
[`FutureExt::map`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.map
[`FutureExt::map_into`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.map_into
[`FutureExt::then`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.then
