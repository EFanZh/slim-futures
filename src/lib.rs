mod assert_future;
mod async_slim_map;
mod fn_mut_1;
mod slim_flatten;
mod slim_future_ext;
mod slim_lazy;
mod slim_map;
mod slim_map_into;
mod slim_ready;

pub mod soon_to_be_pub {
    pub mod foo {
        pub mod bar {
            pub use crate::async_slim_map::AsyncSlimMap;
            pub use crate::slim_flatten::SlimFlatten;
            pub use crate::slim_future_ext::SlimFutureExt;
            pub use crate::slim_lazy::{slim_lazy, SlimLazy};
            pub use crate::slim_map::SlimMap;
            pub use crate::slim_map_into::SlimMapInto;
            pub use crate::slim_ready::{slim_ready, SlimReady};
        }
    }
}
