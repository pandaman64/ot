extern crate ot;

use ot::charwise::*;
use ot::cs::*;
use ot::server;
use ot::server::*;
use ot::client;
use ot::client::*;

use std::rc::Rc;
use std::cell::RefCell;

#[macro_use]
extern crate failure;

extern crate futures;
use futures::Future;

#[test]
fn test_client_server() {

    struct MockConnection(Rc<RefCell<Server>>);

    impl<'a> server::Connection for &'a MockConnection {
        fn send_state(&mut self, _state: &State) {}
    }

    #[derive(Debug, Fail)]
    #[fail(display = "error: {}", _0)]
    struct MockConnectionError(String);

    impl From<String> for MockConnectionError {
        fn from(s: String) -> Self {
            MockConnectionError(s)
        }
    }

    impl client::Connection for MockConnection {
        type Error = MockConnectionError;
        type Output = Box<Future<Item = (Id, Operation), Error = Self::Error>>;
        type StateFuture = Box<Future<Item = State, Error = Self::Error>>;

        fn get_latest_state(&self) -> Self::StateFuture {
            use futures::future::ok;

            let server = self.0.borrow();
            Box::new(ok(server.current_state().clone()))
        }

        fn get_patch_since(&self, since_id: &Id) -> Self::Output {
            use futures::future::result;

            let server = self.0.borrow();
            Box::new(result(server.get_patch(since_id).map_err(Into::into)))
        }

        fn send_operation(&self, parent: Id, op: Operation) -> Self::Output {
            use futures::future::result;

            let mut server = self.0.borrow_mut();
            Box::new(result(server.modify(parent, op).map_err(Into::into)))
        }
    }

    let server = Rc::new(RefCell::new(Server::new()));

    let connection1 = MockConnection(server.clone());
    let connection2 = MockConnection(server.clone());

    server.borrow_mut().connect(Box::new(&connection1));
    server.borrow_mut().connect(Box::new(&connection2));

    let mut client1 = Client::with_connection(&connection1).wait().unwrap();
    let mut client2 = Client::with_connection(&connection2).wait().unwrap();

    assert_eq!(client1.current_content().unwrap(), "");
    assert_eq!(client2.current_content().unwrap(), "");

    client1.push_operation({
        let mut op = Operation::new();
        op.insert("こんにちは 世界".into());
        op
    });
    {
        let (id, op) = client1.send_to_server().unwrap().wait().unwrap();
        client1.apply_patch(id, op).unwrap();
    }

    assert_eq!(client1.current_content().unwrap(), "こんにちは 世界");
    assert_eq!(client2.current_content().unwrap(), "");

    client2.push_operation({
        let mut op = Operation::new();
        op.insert("!".into());
        op
    });
    {
        let (id, op) = client2.send_to_server().unwrap().wait().unwrap();
        client2.apply_patch(id, op).unwrap();
    }

    assert_eq!(client1.current_content().unwrap(), "こんにちは 世界");
    assert_eq!(
        client2.current_content().unwrap(),
        "!こんにちは 世界"
    );

    client1.push_operation({
        let mut op = Operation::new();
        op.delete("こんにちは".len());
        op.insert("さようなら".into());
        op.retain(" 世界".len());
        op
    });
    {
        let (id, op) = client1.send_to_server().unwrap().wait().unwrap();
        client1.apply_patch(id, op).unwrap();
    }

    assert_eq!(
        client1.current_content().unwrap(),
        "!さようなら 世界"
    );
    assert_eq!(
        client2.current_content().unwrap(),
        "!こんにちは 世界"
    );

    {
        let (latest_id, diff) = client2.send_get_patch().wait().unwrap();
        client2.apply_patch(latest_id, diff).unwrap();
    }

    assert_eq!(
        client1.current_content().unwrap(),
        "!さようなら 世界"
    );
    assert_eq!(
        client2.current_content().unwrap(),
        "!さようなら 世界"
    );
}
