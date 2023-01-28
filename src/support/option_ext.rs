use core::ops::ControlFlow;
use core::pin::Pin;

pub trait OptionExt<T> {
    fn insert_pinned(self: Pin<&mut Self>, value: T) -> Pin<&mut T>;

    fn get_or_try_insert_with_pinned<F, B>(self: Pin<&mut Self>, f: F) -> ControlFlow<B, Pin<&mut T>>
    where
        F: FnOnce() -> ControlFlow<B, T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn insert_pinned(mut self: Pin<&mut Self>, value: T) -> Pin<&mut T> {
        self.as_mut().set(Some(value));
        self.as_pin_mut().unwrap()
    }

    fn get_or_try_insert_with_pinned<F, B>(mut self: Pin<&mut Self>, f: F) -> ControlFlow<B, Pin<&mut T>>
    where
        F: FnOnce() -> ControlFlow<B, T>,
    {
        ControlFlow::Continue(if self.as_mut().as_pin_mut().is_none() {
            self.insert_pinned(f()?)
        } else {
            self.as_pin_mut().unwrap()
        })
    }
}
