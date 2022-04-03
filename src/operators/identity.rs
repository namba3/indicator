use crate::{Current, Indicator, Reset};

pub struct Identity<T: Clone>(Option<T>);
impl<T: Clone> Identity<T> {
    pub fn new() -> Self {
        Identity(None)
    }
}
impl<T: Clone> Indicator for Identity<T> {
    type Input = T;
    type Output = T;
    fn next(&mut self, input: T) -> Option<<Self as Indicator>::Output> {
        self.0 = input.into();
        self.0.clone()
    }
}
impl<T: Clone> Current for Identity<T> {
    fn current(&self) -> Option<Self::Output> {
        self.0.clone()
    }
}
impl<T: Clone> Reset for Identity<T> {
    fn reset(&mut self) {
        self.0 = None;
    }
}
