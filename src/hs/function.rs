pub trait Function: Sized {
    fn compl<G, A, B, C>(self, g: G) -> impl Fn(A) -> B
    where
        Self: Fn(C) -> B,
        G: Fn(A) -> C;

    fn compr<G, A, B, C>(self, g: G) -> impl Fn(A) -> B
    where
        Self: Fn(A) -> C,
        G: Fn(C) -> B;
}

impl<F> Function for F
where
    F: Sized,
{
    fn compl<G, A, B, C>(self, g: G) -> impl Fn(A) -> B
    where
        Self: Fn(C) -> B,
        G: Fn(A) -> C,
    {
        move |x| self(g(x))
    }

    fn compr<G, A, B, C>(self, g: G) -> impl Fn(A) -> B
    where
        Self: Fn(A) -> C,
        G: Fn(C) -> B,
    {
        move |x| g(self(x))
    }
}

pub fn id<T>() -> impl Fn(T) -> T {
    |x| x
}
