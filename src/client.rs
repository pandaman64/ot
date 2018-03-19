
use super::*;
use util::*;

extern crate futures;

use self::futures::Future;
use self::futures::FutureExt;

pub trait Connection {
    type Error;
    type Output: Future<Item = (Id, Operation), Error = Self::Error>;

    fn get_latest_state(&self) -> State;
    fn send_operation(&self, base_id: Id, operation: Operation) -> Self::Output;
}

#[derive(Debug)]
pub struct ClientState {
    id: Id,
    content: String,
}

// TODO: change base_id: Id to base_state: State and manage the difference from base_state to the
// current buffer as current_operation
// base_state is the last state we fetched from the server,
// sent_operation is an operation originating from base_state, containing diffs sent to the server
// current_operation is an operation originating from compose(base_state, sent_operation) or
// base_state, which holds diffs between the parent and the current buffer 
pub enum Client<C: Connection> {
    WaitingForResponse {
        base_state: ClientState,
        sent_diff: Operation,
        current_diff: Operation,
        connection: Box<C>,
    },
    Buffering {
        base_state: ClientState,
        current_diff: Operation,
        connection: Box<C>,
    },
    Error(String),
}

impl<C: Connection> Client<C> {
    pub fn with_connection(connection: Box<C>) -> Self {
        let state = connection.get_latest_state();
        Client::Buffering {
            current_diff: {
                let mut op = Operation::new();
                op.retain(state.content.len());
                op
            },
            base_state: ClientState {
                id: state.id,
                content: state.content
            },
            connection: connection,
        }
    }

    pub fn current_content(&self) -> Result<String, String> {
        use self::Client::*;
        match *self {
            WaitingForResponse {
                ref base_state, ..
            } | Buffering {
                ref base_state, ..
            } => {
                Ok(base_state.content.clone())
            },
            Error(ref error) => Err(error.clone()),
        }
    }

    pub fn push_operation(&mut self, operation: Operation) {
        use self::Client::*;
        match *self {
            WaitingForResponse {
                ref mut current_diff, ..
            } => {
                let current = std::mem::replace(current_diff, Operation::new());
                *current_diff = compose(current, operation);
            },
            Buffering {
                ref mut current_diff, ..
            } => {
                let current = std::mem::replace(current_diff, Operation::new());
                *current_diff = compose(current, operation);
            },
            Error(_) => {},
        }
    }

    pub fn send_to_server(&mut self) -> Result<C::Output, String> {
        use self::Client::*;
        if let Buffering { .. } = *self {
            if let Buffering { base_state, current_diff, connection } = std::mem::replace(self, Error("".into())) {
                let ret = connection.send_operation(base_state.id.clone(), current_diff.clone());
                *self = WaitingForResponse {
                    base_state: base_state,
                    current_diff: {
                        let mut op = Operation::new();
                        op.retain(current_diff.target_len());
                        op
                    },
                    sent_diff: current_diff,
                    connection: connection,
                };
                Ok(ret)
            } else {
                unreachable!();
            }
        } else {
            Err("client is not in buffering state".into())
        }
    }

    // this should be impl Future
    pub fn apply_response<'a>(&'a mut self, response: C::Output) -> Box<Future<Item = (), Error = C::Error> + 'a> {
        Box::new(response.map(move |(id, op)| {
            use self::Client::*;
            match std::mem::replace(self, Error("".into())) {
                WaitingForResponse {
                    base_state, sent_diff, current_diff, connection, 
                } => {
                    let content = apply(&base_state.content, &compose(sent_diff, op.clone()));

                    let (current_diff, _) = transform(current_diff, op);

                    *self = Buffering {
                        current_diff: current_diff,
                        base_state: ClientState {
                            id: id,
                            content: content,
                        },
                        connection: connection,
                    };
                },
                _ => unreachable!()
            }
        }))
    }
}

