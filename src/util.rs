use super::Operation;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize, Debug)]
pub struct Id(pub usize);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub parent: Id,
    pub id: Id,
    pub diff: Operation, 
    pub content: String,
}

