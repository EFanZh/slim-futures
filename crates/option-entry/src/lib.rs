#![no_std]

use core::pin::Pin;
use core::ptr;

mod private {
    use core::pin::Pin;

    pub trait Sealed<T> {
        fn borrow_mut(&mut self) -> &mut Option<T>;
        fn borrow_pin_mut(self: Pin<&mut Self>) -> Pin<&mut Option<T>>;
    }

    impl<T> Sealed<T> for Option<T> {
        fn borrow_mut(&mut self) -> &mut Option<T> {
            self
        }

        fn borrow_pin_mut(self: Pin<&mut Self>) -> Pin<&mut Option<T>> {
            self
        }
    }
}

fn write_without_calling_destructor<T>(target: &mut T, value: T) {
    // SAFETY: Not calling a destructor is safe.
    unsafe {
        ptr::write(target, value);
    }
}

fn write_pin_mut_without_calling_destructor<T>(target: Pin<&mut T>, value: T) {
    // SAFETY: We are not moving the old value, and not calling a destructor is safe.
    unsafe {
        ptr::write(Pin::get_unchecked_mut(target), value);
    }
}

pub struct OptionNoneEntry<'a, T> {
    /// `inner` must be in `None` state.
    inner: &'a mut Option<T>,
}

impl<'a, T> OptionNoneEntry<'a, T> {
    pub fn replace_some(self, value: T) -> OptionSomeEntry<'a, T> {
        let inner = self.inner;

        // `OptionSomeEntry` proves that `inner` is in `None` state, so destructor can be safely ignored.
        write_without_calling_destructor(inner, Some(value));

        OptionSomeEntry { inner }
    }
}

pub struct OptionSomeEntry<'a, T> {
    /// `inner` must be in `Some` state.
    inner: &'a mut Option<T>,
}

impl<'a, T> OptionSomeEntry<'a, T> {
    pub fn get_mut(&mut self) -> &mut T {
        // SAFETY: `OptionSomeEntry` proves that `self.inner` is in `Some` state.
        unsafe { self.inner.as_mut().unwrap_unchecked() }
    }

    pub fn into_mut(self) -> &'a mut T {
        // SAFETY: `OptionSomeEntry` proves that `self.inner` is in `Some` state.
        unsafe { self.inner.as_mut().unwrap_unchecked() }
    }

    pub fn replace_none(self) -> (OptionNoneEntry<'a, T>, T) {
        let inner = self.inner;

        // SAFETY: `OptionSomeEntry` proves that `inner` is in `Some` state.
        let value = unsafe { inner.take().unwrap_unchecked() };

        (OptionNoneEntry { inner }, value)
    }
}

pub enum OptionEntry<'a, T> {
    None(OptionNoneEntry<'a, T>),
    Some(OptionSomeEntry<'a, T>),
}

pub struct OptionNonePinnedEntry<'a, T> {
    /// `inner` must be in `None` state.
    inner: Pin<&'a mut Option<T>>,
}

impl<'a, T> OptionNonePinnedEntry<'a, T> {
    pub fn replace_some(self, value: T) -> OptionSomePinnedEntry<'a, T> {
        let mut inner = self.inner;

        // `OptionSomePinnedEntry` proves that `inner` is in `None` state, so destructor can be safely ignored.
        write_pin_mut_without_calling_destructor(inner.as_mut(), Some(value));

        OptionSomePinnedEntry { inner }
    }
}

pub struct OptionSomePinnedEntry<'a, T> {
    /// `inner` must be in `Some` state.
    inner: Pin<&'a mut Option<T>>,
}

impl<'a, T> OptionSomePinnedEntry<'a, T> {
    pub fn get_pin_mut(&mut self) -> Pin<&mut T> {
        // SAFETY: `OptionSomePinnedEntry` proves that `self.inner` is in `Some` state.
        unsafe { self.inner.as_mut().as_pin_mut().unwrap_unchecked() }
    }

    pub fn into_pin_mut(self) -> Pin<&'a mut T> {
        // SAFETY: `OptionSomePinnedEntry` proves that `self.inner` is in `Some` state.
        unsafe { self.inner.as_pin_mut().unwrap_unchecked() }
    }

    pub fn replace_none(self) -> OptionNonePinnedEntry<'a, T> {
        let mut inner = self.inner;

        {
            let mut replace_none_on_drop = scopeguard::guard(inner.as_mut(), |inner| {
                //  Value destructor have already been called, we are not calling it again.
                write_pin_mut_without_calling_destructor(inner, None)
            });

            unsafe {
                // SAFETY: We are not moving the value, and we are setting the value to `None` afterwards so the
                // destructor will not be called more than once.
                ptr::drop_in_place(Pin::get_unchecked_mut(
                    replace_none_on_drop.as_mut().as_pin_mut().unwrap_unchecked(),
                ));
            }
        }

        OptionNonePinnedEntry { inner }
    }

    pub fn replace_some(&mut self, value: T) {
        self.get_pin_mut().set(value);
    }
}

pub enum OptionPinnedEntry<'a, T> {
    None(OptionNonePinnedEntry<'a, T>),
    Some(OptionSomePinnedEntry<'a, T>),
}

pub trait OptionEntryExt<T>: private::Sealed<T> {
    fn entry(&mut self) -> OptionEntry<T> {
        let inner = self.borrow_mut();

        match inner {
            None => OptionEntry::None(OptionNoneEntry { inner }),
            Some(..) => OptionEntry::Some(OptionSomeEntry { inner }),
        }
    }

    fn pinned_entry(self: Pin<&mut Self>) -> OptionPinnedEntry<T> {
        let mut inner = self.borrow_pin_mut();

        match inner.as_mut().as_pin_mut() {
            None => OptionPinnedEntry::None(OptionNonePinnedEntry { inner }),
            Some(..) => OptionPinnedEntry::Some(OptionSomePinnedEntry { inner }),
        }
    }
}

impl<T> OptionEntryExt<T> for Option<T> {}
