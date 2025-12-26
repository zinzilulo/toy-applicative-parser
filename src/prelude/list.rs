use crate::prelude::{Applicative, Functor, Monad};

impl<'a, T> Functor<'a> for Vec<T> {
    type Wrapped<U>
        = Vec<U>
    where
        U: 'a;

    fn fmap<A, B, F>(fa: &Vec<A>, f: F) -> Vec<B>
    where
        A: Clone,
        F: Fn(A) -> B + 'a,
    {
        fa.iter().cloned().map(f).collect()
    }
}

impl<'a, T> Applicative<'a> for Vec<T> {
    fn pure<A>(a: &A) -> Vec<A>
    where
        A: Clone + 'a,
    {
        vec![a.clone()]
    }

    fn ap<X, B, FFn>(fa: &Vec<X>, fab: Vec<FFn>) -> Vec<B>
    where
        X: Clone + 'a,
        FFn: Fn(X) -> B + 'a,
    {
        let mut out = Vec::new();
        for f in fab {
            for x in fa.iter().cloned() {
                out.push(f(x));
            }
        }
        out
    }
}

impl<'a, T> Monad<'a> for Vec<T> {
    fn bind<A, B, K>(ma: &Vec<A>, k: K) -> Vec<B>
    where
        A: Clone + 'a,
        K: Fn(A) -> Vec<B> + 'a,
    {
        let mut out = Vec::new();
        for a in ma.iter().cloned() {
            out.extend(k(a));
        }
        out
    }
}

impl<'a, T, const N: usize> Functor<'a> for [T; N] {
    type Wrapped<U>
        = [U; N]
    where
        U: 'a;

    fn fmap<A, B, F>(fa: &[A; N], f: F) -> [B; N]
    where
        A: Clone,
        F: Fn(A) -> B + 'a,
    {
        std::array::from_fn(|i| f(fa[i].clone()))
    }
}

impl<'a, T, const N: usize> Applicative<'a> for [T; N] {
    fn pure<A>(a: &A) -> [A; N]
    where
        A: Clone + 'a,
    {
        std::array::from_fn(|_| a.clone())
    }

    fn ap<X, B, FFn>(fa: &[X; N], fab: [FFn; N]) -> [B; N]
    where
        X: Clone + 'a,
        FFn: Fn(X) -> B + 'a,
    {
        std::array::from_fn(|i| fab[i](fa[i].clone()))
    }
}

impl<'a, T> Monad<'a> for [T; 1] {
    fn bind<A, B, K>(ma: &[A; 1], k: K) -> [B; 1]
    where
        A: Clone + 'a,
        K: Fn(A) -> [B; 1] + 'a,
    {
        let a0 = ma[0].clone();
        k(a0)
    }
}
