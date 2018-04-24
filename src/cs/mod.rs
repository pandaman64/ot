use super::Operation;

pub mod server;
pub mod client;
pub mod mock_connection;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize, Debug)]
pub struct Id(pub usize);

#[derive(Clone, Debug)]
pub struct State<O: Operation> {
    pub parent: Id,
    pub id: Id,
    pub diff: O,
    pub content: O::Target,
}
