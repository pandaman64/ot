
use super::*;
use util::*;

extern crate futures;

use self::futures::Future;
use self::futures::FutureExt;

pub trait Connection {
    type Error;
    type Output: Future<Item = (Id, Operation), Error = Self::Error>;

    fn get_latest_state(&self) -> (Id, Operation);
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
        base_id: Id,
        sent_operation: Operation,
        current_operation: Operation,
        connection: Box<C>,
    },
    Buffering {
        base_id: Id,
        current_operation: Operation,
        connection: Box<C>,
    },
    Error(String),
}

impl<C: Connection> Client<C> {
    pub fn with_connection(connection: Box<C>) -> Self {
        let (base_id, current) = connection.get_latest_state();
        Client::Buffering {
            base_id: base_id,
            current_operation: current,
            connection: connection,
        }
    }

    pub fn current_content(&self) -> Result<String, String> {
        use self::Client::*;
        match *self {
            WaitingForResponse {
                ref sent_operation, ref current_operation, ..
            } => {
                Ok(apply("", &compose(sent_operation.clone(), current_operation.clone())))
            },
            Buffering {
                ref current_operation, ..
            } => {
                Ok(apply("", current_operation))
            },
            Error(ref error) => Err(error.clone()),
        }
    }

    pub fn push_operation(&mut self, operation: Operation) {
        use self::Client::*;
        match *self {
            WaitingForResponse {
                ref mut current_operation, ..
            } => {
                let current = std::mem::replace(current_operation, Operation::new());
                *current_operation = compose(current, operation);
            },
            Buffering {
                ref mut current_operation, ..
            } => {
                let current = std::mem::replace(current_operation, Operation::new());
                *current_operation = compose(current, operation);
            },
            Error(_) => {},
        }
    }

    pub fn send_to_server(&mut self) -> Result<C::Output, String> {
        use self::Client::*;
        if let Buffering { .. } = *self {
            if let Buffering { base_id, current_operation, connection } = std::mem::replace(self, Error("".into())) {
                let ret = connection.send_operation(base_id.clone(), current_operation.clone());
                *self = WaitingForResponse {
                    base_id: base_id,
                    current_operation: {
                        let mut op = Operation::new();
                        op.retain(current_operation.target_len());
                        op
                    },
                    sent_operation: current_operation,
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
                    sent_operation, current_operation, connection, ..
                } => {
                    let (current_, _) = transform(current_operation, op.clone());
                    
                    *self = Buffering {
                        base_id: id,
                        current_operation: compose(compose(sent_operation, op), current_),
                        connection: connection,
                    };
                },
                _ => unreachable!()
            }
        }))
    }
}

