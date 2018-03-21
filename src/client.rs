
use std::marker::PhantomData;

use super::*;
use util::*;

extern crate futures;

use self::futures::Future;
use self::futures::FutureExt;

pub trait Connection {
    type Error;
    type Output: Future<Item = (Id, Operation), Error = Self::Error>;
    type StateFuture: Future<Item = State, Error = Self::Error>;

    fn get_latest_state(&self) -> Self::StateFuture;
    fn send_operation(&self, base_id: Id, operation: Operation) -> Self::Output;
}

#[derive(Debug)]
pub struct ClientState {
    id: Id,
    content: String,
}

#[derive(Debug)]
pub enum ClientError<'a, E> {
    ConnectionError(E),
    OutOfDate,
    NotConnected(&'a str),
}

pub enum Client<'c, C: Connection + 'c> {
    WaitingForResponse {
        base_state: ClientState,
        sent_diff: Operation,
        current_diff: Option<Operation>,
        connection: Box<C>,
    },
    Buffering {
        base_state: ClientState,
        current_diff: Option<Operation>,
        connection: Box<C>,
    },
    Error(String, PhantomData<&'c ()>),
}

impl<'c, C: Connection + 'c> Client<'c, C> {
    pub fn with_connection(connection: Box<C>) -> Box<Future<Item = Self, Error = C::Error> + 'c> {
        Box::new(connection.get_latest_state()
            .map(|state| 
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
            Error(ref error, _) => Err(error.clone()),
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
            Error(_, _) => {},
        }
    }

    pub fn send_to_server(&mut self) -> Result<C::Output, String> {
        use self::Client::*;
        if let &mut Buffering { current_diff: Some(_), .. } = self {
            if let Buffering { base_state, current_diff, connection } = std::mem::replace(self, Error("".into(), PhantomData)) {
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

    // this should be impl Future
    pub fn apply_response<'a>(&'a mut self, response: C::Output) -> Box<Future<Item = (), Error = C::Error> + 'a> {
        Box::new(response.map(move |(id, op)| {
            use self::Client::*;
            match std::mem::replace(self, Error("".into(), PhantomData)) {
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
                },
                _ => unreachable!()
            }
        }))
    }

    pub fn update<'a>(&'a mut self) -> Box<Future<Item = (), Error = ClientError<'a, C::Error>> + 'a> {
        use self::Client::*;
        use self::ClientError::*;
        use self::futures::future::{ok, err};

        match *self {
            Error(ref s, _) => Box::new(err(NotConnected(s))),
            WaitingForResponse { .. } => Box::new(err(OutOfDate)),
            Buffering {
                ref mut base_state,
                ref mut current_diff,
                ref mut connection,
            } => {
                Box::new(connection.get_latest_state()
                    .map_err(ConnectionError) // should we change self to Error?
                    .and_then(move |state| 
                        if base_state.id == state.parent {
                            if let Some(current) = std::mem::replace(current_diff, None) {
                                *current_diff = Some(transform(current, state.diff).0);
                            }
                            *base_state = ClientState {
                                id: state.id,
                                content: state.content
                            };
                            Box::new(ok(()))
                        } else {
                            Box::new(err(OutOfDate))
                        }))
            },
        }
    }
}

