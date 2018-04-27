use super::Operation;

pub mod server;
pub mod client;
pub mod mock_connection;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize, Debug)]
pub struct Id(pub usize);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State<O: Operation> {
    pub parent: Id,
    pub id: Id,
    #[serde(bound(serialize = "O: Serialize", deserialize = "O: Deserialize<'de>"))]
    pub diff: O,
    #[serde(bound(serialize = "O::Target: Serialize",
                  deserialize = "O::Target: Deserialize<'de>"))]
    pub content: O::Target,
}
