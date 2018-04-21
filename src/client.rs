
use super::*;
use util::*;
use super::charwise::{apply, compose, transform, Operation};

extern crate failure;
use failure::{Error, Fail};

extern crate futures;
use self::futures::Future;

pub trait Connection {
    type Error: Fail;
    type Output: Future<Item = (Id, Operation), Error = Self::Error> + 'static;
    type StateFuture: Future<Item = State, Error = Self::Error> + 'static;

    fn get_latest_state(&self) -> Self::StateFuture;
    fn get_patch_since(&self, since_id: &Id) -> Self::Output;
    fn send_operation(&self, base_id: Id, operation: Operation) -> Self::Output;
}

impl<C: Connection + ?Sized> Connection for Box<C> {
    type Error = C::Error;
    type Output = C::Output;
    type StateFuture = C::StateFuture;

    fn get_latest_state(&self) -> Self::StateFuture {
        (**self).get_latest_state()
    }

    fn get_patch_since(&self, since_id: &Id) -> Self::Output {
        (**self).get_patch_since(since_id)
    }

    fn send_operation(&self, base_id: Id, operation: Operation) -> Self::Output {
        (**self).send_operation(base_id, operation)
    }
}

impl<'c, C: Connection + ?Sized + 'c> Connection for &'c C {
    type Error = C::Error;
    type Output = C::Output;
    type StateFuture = C::StateFuture;

    fn get_latest_state(&self) -> Self::StateFuture {
        (*self).get_latest_state()
    }

    fn get_patch_since(&self, since_id: &Id) -> Self::Output {
        (*self).get_patch_since(since_id)
    }

    fn send_operation(&self, base_id: Id, operation: Operation) -> Self::Output {
        (*self).send_operation(base_id, operation)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientState {
    id: Id,
    content: String,
}

#[derive(Debug, Fail)]
pub enum ClientError {
    #[fail(display = "Error occured in connection: {}", _0)]
    ConnectionError(Error),
    #[fail(display = "Invalid operation on syncing state")]
    Syncing,
    #[fail(display = "Client not connected any more: {}", _0)]
    NotConnected(String),
}

pub enum Client<C: Connection> {
    WaitingForResponse {
        base_state: ClientState,
        sent_diff: Operation,
        current_diff: Option<Operation>,
        connection: C,
    },
    Buffering {
        base_state: ClientState,
        current_diff: Option<Operation>,
        connection: C,
    },
    Error(String),
}

impl<'c, C: Connection + 'c> Client<C> {
    pub fn with_connection(connection: C) -> Box<Future<Item = Self, Error = C::Error> + 'c> {
        Box::new(connection.get_latest_state()
            .map(move |state| 
                Client::Buffering {
                    current_diff: None,
                    base_state: ClientState {
                        id: state.id,
                        content: state.content
                    },
                    connection: connection,
                }))
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
            } | Buffering {
                ref mut current_diff, ..
            } => {
                if let Some(current) = std::mem::replace(current_diff, None) {
                    *current_diff = Some(compose(current, operation));
                } else {
                    *current_diff = Some(operation);
                }
            },
            Error(_) => {},
        }
    }

    pub fn send_to_server(&mut self) -> Result<C::Output, String> {
        use self::Client::*;
        if let &mut Buffering { current_diff: Some(_), .. } = self {
            if let Buffering { base_state, current_diff, connection } = std::mem::replace(self, Error("".into())) {
                let current_diff = current_diff.unwrap();
                let ret = connection.send_operation(base_state.id.clone(), current_diff.clone());
                *self = WaitingForResponse {
                    base_state: base_state,
                    current_diff: None,
                    sent_diff: current_diff,
                    connection: connection,
                };
                Ok(ret)
            } else {
                unreachable!();
            }
        } else if let Buffering { .. } = *self {
            Err("client has no diff in buffer".into())
        } else {
            Err("client is not in buffering state".into())
        }
    }

    pub fn apply_patch(&mut self, latest_id: Id, diff: Operation) -> Result<(), ClientError> {
        use self::Client::*;
        use self::ClientError::*;

        match std::mem::replace(self, Error("".into())) {
            Error(ref s) => Err(NotConnected(s.clone())),
            WaitingForResponse {
                base_state, sent_diff, current_diff, connection, 
            } => {
                let content = apply(&base_state.content, &compose(sent_diff, diff.clone()));

                *self = Buffering {
                    current_diff: current_diff.map(|current| transform(current, diff).0),
                    base_state: ClientState {
                        id: latest_id,
                        content: content,
                    },
                    connection: connection,
                };

                Ok(())
            },
            Buffering {
                mut base_state,
                mut current_diff,
                connection, 
            } => {
                Self::patch(&mut base_state, &mut current_diff, latest_id, diff)?;
                *self = Buffering { base_state, current_diff, connection };
                Ok(())
            },
        }
    }

    pub fn apply_response(&mut self, id: Id, op: Operation) -> Result<(), C::Error> {
        use self::Client::*;
        match std::mem::replace(self, Error("".into())) {
            WaitingForResponse {
                base_state, sent_diff, current_diff, connection, 
            } => {
                let content = apply(&base_state.content, &compose(sent_diff, op.clone()));

                *self = Buffering {
                    current_diff: current_diff.map(|diff| transform(diff, op).0),
                    base_state: ClientState {
                        id: id,
                        content: content,
                    },
                    connection: connection,
                };

                Ok(())
            },
            _ => unreachable!()
        }
    }

    fn patch<'a>(base_state: &'a mut ClientState, current_diff: &'a mut Option<Operation>, latest_id: Id, diff: Operation) -> Result<(), ClientError> {
        let content;
        if let Some(current) = std::mem::replace(current_diff, None) {
            let (current, _) = transform(current, diff.clone());
            content = apply(&base_state.content, &compose(diff, current));
        } else {
            content = apply(&base_state.content, &diff);
        }

        *base_state = ClientState {
            id: latest_id,
            content: content,
        };

        Ok(())
    }

    pub fn send_get_patch(&self) -> Box<Future<Item = (Id, Operation), Error = ClientError>> {
        use self::Client::*;
        use self::ClientError::*;
        use self::futures::future::err;

        match *self {
            Error(ref s) => Box::new(err(NotConnected(s.clone()))),
            WaitingForResponse { .. } => Box::new(err(Syncing)),
            Buffering {
                ref base_state,
                ref connection, 
                ..
            } => {
                Box::new(connection.get_patch_since(&base_state.id)
                    .map_err(Into::into)
                    .map_err(ConnectionError)) // should we change self to Error?
            },
        }
    }
}

