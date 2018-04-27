extern crate ot;

use ot::charwise::*;
use ot::cs::*;
use ot::server::*;
use ot::client::*;

use std::rc::Rc;
use std::cell::RefCell;

extern crate failure;

extern crate futures;
use futures::executor::block_on;

#[test]
fn test_charwise_client_server() {
    let server = Rc::new(RefCell::new(Server::new()));

    let mut connection1 = mock_connection::MockConnection::new(server.clone());
    let mut connection2 = mock_connection::MockConnection::new(server.clone());

    server.borrow_mut().connect(Box::new(&mut connection1));
    server.borrow_mut().connect(Box::new(&mut connection2));

    let mut client1 = block_on(Client::with_connection(&connection1)).unwrap();
    let mut client2 = block_on(Client::with_connection(&connection2)).unwrap();

    assert_eq!(client1.current_content().unwrap(), "");
    assert_eq!(client2.current_content().unwrap(), "");

    client1.push_operation({
        let mut op = Operation::new();
        op.insert("こんにちは 世界".into());
        op
    });
    {
        let (id, op) = block_on(client1.send_to_server().unwrap()).unwrap();
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
        let (id, op) = block_on(client2.send_to_server().unwrap()).unwrap();
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
        let (id, op) = block_on(client1.send_to_server().unwrap()).unwrap();
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
        let (latest_id, diff) = block_on(client2.send_get_patch()).unwrap();
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
