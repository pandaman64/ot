use std::default::Default;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate failure;

pub mod cs;
pub use cs::server;
pub use cs::client;
pub mod charwise;
pub mod linewise;
pub mod selection;

pub trait Operation: Sized + Default + Clone {
    type Target: Default + Clone;

    // return an operation does nothing when applied to target
    fn nop(target: &Self::Target) -> Self;

    // apply operation to target
    fn apply(&self, target: &Self::Target) -> Self::Target;

    // compose two operations
    // compose must satisfy apply(apply(s, a), b) == apply(s, compose(a, b))
    fn compose(self, other: Self) -> Self;

    // transforms two operations so that composed operations will converge
    // let (left', right') = transform(left, right), these satisfies the condition
    // apply(s, compose(left, right')) == apply(s, compose(right, left'))
    fn transform(self, other: Self) -> (Self, Self);
}
