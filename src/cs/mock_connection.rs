use Operation;
use super::{Id, State};
use super::server;
use server::Server;
use super::client;

use futures::Future;

use std::rc::Rc;
use std::cell::RefCell;

pub struct MockConnection<O: Operation>(Rc<RefCell<Server<O>>>);

impl<O: Operation> MockConnection<O> {
    pub fn new(server: Rc<RefCell<Server<O>>>) -> Self {
        MockConnection(server)
    }
}

impl<O: Operation> server::Connection<O> for MockConnection<O> {
    fn send_state(&mut self, _state: &State<O>) {}
}

#[derive(Debug, Fail)]
#[fail(display = "error: {}", _0)]
pub struct MockConnectionError(String);

impl From<String> for MockConnectionError {
    fn from(s: String) -> Self {
        MockConnectionError(s)
    }
}

impl<O: Operation + 'static> client::Connection<O> for MockConnection<O> {
    type Error = MockConnectionError;
    type Output = Box<Future<Item = (Id, O), Error = Self::Error>>;
    type StateFuture = Box<Future<Item = State<O>, Error = Self::Error>>;

    fn get_latest_state(&self) -> Self::StateFuture {
        use futures::future::ok;

        let server = self.0.borrow();
        Box::new(ok((*server.current_state()).clone()))
    }

    fn get_patch_since(&self, since_id: &Id) -> Self::Output {
        use futures::future::result;

        let server = self.0.borrow();
        Box::new(result(server.get_patch(since_id).map_err(Into::into)))
    }

    fn send_operation(&self, parent: Id, op: O) -> Self::Output {
        use futures::future::result;

        let mut server = self.0.borrow_mut();
        Box::new(result(server.modify(parent, op).map_err(Into::into)))
    }
}
