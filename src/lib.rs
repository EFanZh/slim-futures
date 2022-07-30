mod fn_mut_1;
mod slim_flatten;
mod slim_future_ext;
mod slim_map;
mod slim_map_async;

pub mod soon_to_be_pub {
    pub mod foo {
        pub mod bar {
            pub use crate::slim_future_ext::SlimFutureExt;
        }
    }
}
