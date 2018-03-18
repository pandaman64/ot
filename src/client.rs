
use super::*;
use util::*;

extern crate futures;

use self::futures::Future;
use self::futures::FutureExt;

pub trait Connection {
    type Error;
    type Output: Future<Item = (Id, Id, Operation), Error = Self::Error>;

    fn get_latest_state(&self) -> State;
    fn send_operation(&self, base_id: Id, operation: Operation) -> Self::Output;
}

// TODO: change base_id: Id to base_state: State and manage the difference from base_state to the
// current buffer as current_operation
// base_state is the last state we fetched from the server,
// sent_operation is an operation originating from base_state, containing diffs sent to the server
// current_operation is an operation originating from compose(base_state, sent_operation) or
// base_state, which holds diffs between the parent and the current buffer 
pub enum Client<C: Connection> {
    WaitingForResponse {
        base_state: State,
        sent_diff: Operation,
        current_diff: Operation,
        connection: Box<C>,
    },
    Buffering {
        base_state: State,
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
                op.retain(state.operation.target_len());
                op
            },
            base_state: state,
            connection: connection,
        }
    }

    pub fn current_content(&self) -> Result<String, String> {
        use self::Client::*;
        match *self {
            WaitingForResponse {
                ref base_state, ref sent_diff, ref current_diff, ..
            } => {
                Ok(apply("", &compose(base_state.operation.clone(), compose(sent_diff.clone(), current_diff.clone()))))
            },
            Buffering {
                ref base_state, ref current_diff, ..
            } => {
                Ok(apply("", &compose(base_state.operation.clone(), current_diff.clone())))
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
        Box::new(response.map(move |(parent_id, id, op)| {
            use self::Client::*;
            match std::mem::replace(self, Error("".into())) {
                WaitingForResponse {
                    sent_diff, current_diff, connection, ..
                } => {
                    let (current_, _) = transform(current_diff, op.clone());
                    let current_operation = compose(compose(sent_diff, op), current_);

                    *self = Buffering {
                        current_diff: {
                            let mut op = Operation::new();
                            op.retain(current_operation.target_len());
                            op
                        },
                        base_state: State {
                            parent: parent_id,
                            id: id,
                            operation: current_operation,
                        },
                        connection: connection,
                    };
                },
                _ => unreachable!()
            }
        }))
    }
}

