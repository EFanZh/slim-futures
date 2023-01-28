use core::pin::Pin;

pub trait OptionExt<T> {
    fn insert_pinned(self: Pin<&mut Self>, value: T) -> Pin<&mut T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn insert_pinned(mut self: Pin<&mut Self>, value: T) -> Pin<&mut T> {
        self.as_mut().set(Some(value));
        self.as_pin_mut().unwrap()
    }
}
