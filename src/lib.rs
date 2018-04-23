#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate failure;

pub mod util;
pub mod server;
pub mod client;
pub mod charwise;
pub mod linewise;
pub mod selection;

pub trait Operation: Sized + std::default::Default {
    type Target;

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

