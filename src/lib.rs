mod assert_future;
mod fn_mut_1;
mod slim_and_then;
mod slim_flatten;
mod slim_future_ext;
mod slim_inspect;
mod slim_lazy;
mod slim_map;
mod slim_map_async;
mod slim_map_into;
mod slim_map_ok;
mod slim_ready;
mod slim_select;
mod slim_try_flatten;

pub mod zzz_you_shall_not_use {
    pub mod foo {
        pub mod bar {
            pub use crate::slim_and_then::SlimAndThen;
            pub use crate::slim_flatten::SlimFlatten;
            pub use crate::slim_future_ext::SlimFutureExt;
            pub use crate::slim_inspect::SlimInspect;
            pub use crate::slim_lazy::{slim_lazy, SlimLazy};
            pub use crate::slim_map::SlimMap;
            pub use crate::slim_map_async::SlimMapAsync;
            pub use crate::slim_map_into::SlimMapInto;
            pub use crate::slim_map_ok::SlimMapOk;
            pub use crate::slim_ready::{slim_ready, SlimReady};
            pub use crate::slim_select::SlimSelect;
            pub use crate::slim_try_flatten::SlimTryFlatten;
        }
    }
}
