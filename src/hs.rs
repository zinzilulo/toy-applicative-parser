mod alternative;
mod applicative;
mod function;
mod functor;
mod hlist;
mod maybe;
mod monad;

pub use crate::hs::alternative::Alternative;
pub use crate::hs::applicative::Applicative;
pub use crate::hs::function::{Function, id};
pub use crate::hs::functor::Functor;
pub use crate::hs::maybe::{Just, Maybe, Nothing, catMaybes};
pub use crate::hs::monad::Monad;

fn foldr<T, U, F>(list: &[T], init: U, f: F) -> U
where
    T: Clone,
    F: Fn(T, U) -> U + Copy,
{
    match list.split_first() {
        Some((head, tail)) => f(head.clone(), foldr(tail, init, f)),
        None => init,
    }
}
