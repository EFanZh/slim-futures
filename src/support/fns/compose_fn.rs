use fn_traits::FnMut;

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

impl<T, F, G> FnMut<(T,)> for ComposeFn<F, G>
where
    F: FnMut<(T,)>,
    G: FnMut<(F::Output,)>,
{
    type Output = G::Output;

    fn call_mut(&mut self, args: (T,)) -> Self::Output {
        self.second.call_mut((self.first.call_mut(args),))
    }
}
