pin_project_lite::pin_project! {
    pub struct PinnedAndNotPinned<A, B> {
        #[pin]
        pub pinned: A,
        pub not_pinned: B,
    }
}

impl<A, B> PinnedAndNotPinned<A, B> {
    pub fn new(pinned: A, not_pinned: B) -> Self {
        Self { pinned, not_pinned }
    }
}

// Manual implement `Clone` to avoid inlining.
impl<A, B> Clone for PinnedAndNotPinned<A, B>
where
    A: Clone,
    B: Clone,
{
    fn clone(&self) -> Self {
        Self {
            pinned: self.pinned.clone(),
            not_pinned: self.not_pinned.clone(),
        }
    }
}
