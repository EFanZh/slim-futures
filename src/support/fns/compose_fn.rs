use crate::support::fn_mut_1::FnMut1;

pub struct ComposeFn<F, G> {
    first: F,
    second: G,
}

impl<F, G> ComposeFn<F, G> {
    pub fn new(first: F, second: G) -> Self {
        Self { first, second }
    }
}

impl<F, G> Clone for ComposeFn<F, G>
where
    F: Clone,
    G: Clone,
{
    fn clone(&self) -> Self {
        Self {
            first: self.first.clone(),
            second: self.second.clone(),
        }
    }
}

impl<T, F, G> FnMut1<T> for ComposeFn<F, G>
where
    F: FnMut1<T>,
    G: FnMut1<F::Output>,
{
    type Output = G::Output;

    fn call_mut(&mut self, arg: T) -> Self::Output {
        self.second.call_mut(self.first.call_mut(arg))
    }
}
