mod alternative;
mod applicative;
mod function;
mod functor;
mod hlist;
mod maybe;
mod monad;

pub use crate::prelude::alternative::Alternative;
pub use crate::prelude::alternative::{alt, alt_with, empty, many, optional, some};
pub use crate::prelude::applicative::Applicative;
pub use crate::prelude::applicative::{
    liftA2, pure, sequenceA, then_keep_left, then_keep_right, traverse,
};
pub use crate::prelude::function::{Function, id};
pub use crate::prelude::functor::Functor;
pub use crate::prelude::functor::fmap;
pub use crate::prelude::maybe::{Just, Maybe, Nothing, catMaybes};
pub use crate::prelude::monad::Monad;
pub use crate::prelude::monad::{bind, join};

fn foldr<T, U, F>(list: &[T], init: U, f: F) -> U
where
    T: Clone,
    F: Fn(T, U) -> U + Copy,
{
    match list.split_first() {
        Just((head, tail)) => f(head.clone(), foldr(tail, init, f)),
        Nothing => init,
    }
}
