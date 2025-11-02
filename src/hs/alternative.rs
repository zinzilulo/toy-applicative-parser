use crate::hs::applicative::Applicative;

pub trait Alternative<'a>: Applicative<'a> {
    fn empty() -> Self
    where
        Self: Sized;

    fn alt(&self, fb: &Self) -> Self
    where
        Self: Sized;

    fn alt_with<F>(&self, fb: F) -> Self
    where
        Self: Sized,
        F: FnOnce() -> Self;

    fn optional(&self) -> Self::Wrapped<Option<Self::Inner>>
    where
        Self: Sized,
        Self::Inner: Clone + 'static;

    fn many(&self) -> Self::Wrapped<Vec<Self::Inner>>
    where
        Self: Sized,
        Self::Inner: Clone + 'static;

    fn some(&self) -> Self::Wrapped<Vec<Self::Inner>>
    where
        Self: Sized,
        Self::Inner: Clone + 'static;
}
