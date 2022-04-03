use crate::{Current, Indicator, Reset};

pub struct Constant<T: Clone>(T);
impl<T: Clone> From<T> for Constant<T> {
    fn from(t: T) -> Self {
        Self(t)
    }
}
impl<T: Clone> Indicator for Constant<T> {
    type Input = ();
    type Output = T;
    fn next(&mut self, _input: ()) -> Option<<Self as Indicator>::Output> {
        self.0.clone().into()
    }
}
impl<T: Clone> Current for Constant<T> {
    fn current(&self) -> Option<Self::Output> {
        self.0.clone().into()
    }
}
impl<T: Clone> Reset for Constant<T> {
    fn reset(&mut self) {}
}
