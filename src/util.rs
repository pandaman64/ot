use super::Operation;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
pub struct Id(pub usize);

#[derive(Clone, Debug)]
pub struct State {
    pub parent: Id,
    pub operation: Operation, 
}

