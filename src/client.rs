
use super::*;
use util::*;

trait Future {
    type Item;
    type Error;
}

pub struct ServerOperation {
}

impl Future for ServerOperation {
    type Item = Operation;
    type Error = ();
}

pub trait Connection {
    fn get_latest_state(&self) -> (Id, Operation);
    fn send_operation(&self, operation: Operation) -> ServerOperation;
}

pub enum Client {
    WaitingForResponse {
        base_id: Id,
        sent_operation: Operation,
        current_operation: Operation,
        connection: Box<Connection>,
    },
    Buffering {
        base_id: Id,
        current_operation: Operation,
        connection: Box<Connection>,
    },
    Error(String),
}

impl Client {
    pub fn with_connection(connection: Box<Connection>) -> Self {
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

    pub fn send_to_server(&mut self) {
        use self::Client::*;
        if let Buffering { .. } = *self {
            if let Buffering { base_id, current_operation, connection } = std::mem::replace(self, Error("".into())) {
                // TODO: do something
                connection.send_operation(current_operation.clone());
                *self = WaitingForResponse {
                    base_id: base_id,
                    sent_operation: current_operation,
                    current_operation: Operation::new(),
                    connection: connection,
                }
            } else {
                unreachable!();
            }
        }
    }
}

