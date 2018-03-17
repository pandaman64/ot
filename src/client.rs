
use super::*;
use util::*;

extern crate futures;

use self::futures::Future;
use self::futures::FutureExt;

pub trait Connection {
    type Error;
    type Output: Future<Item = (Id, Operation), Error = Self::Error>;

    fn get_latest_state(&self) -> (Id, Operation);
    fn send_operation(&self, operation: Operation) -> Self::Output;
}

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

    pub fn send_to_server(&mut self) -> Result<C::Output, ()> {
        use self::Client::*;
        if let Buffering { .. } = *self {
            if let Buffering { base_id, current_operation, connection } = std::mem::replace(self, Error("".into())) {
                let ret = connection.send_operation(current_operation.clone());
                *self = WaitingForResponse {
                    base_id: base_id,
                    sent_operation: current_operation,
                    // this might have to be Operation::new().retain(sent_operation.target_len())
                    // because this is the operation that satisfies compose(sent, current) == sent 
                    current_operation: Operation::new(),
                    connection: connection,
                };
                Ok(ret)
            } else {
                unreachable!();
            }
        } else {
            Err(())
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

