pub trait Functor {
    type Inner;
    type Wrapped<B>;

    fn fmap<B, F>(self, f: F) -> Self::Wrapped<B>
    where
        F: Fn(Self::Inner) -> B + 'static;
}
