extern crate ot;

use ot::selection::linewise::*;
use ot::cs::*;
use ot::server::*;
use ot::client::*;

use std::rc::Rc;
use std::cell::RefCell;
use std::default::Default;

extern crate failure;

extern crate futures;
use futures::Future;

#[test]
fn test_linewise_selection_client_server() {
    use Selection::*;
    use ot::linewise::Operation as BaseOperation;

    let server = Rc::new(RefCell::new(Server::new()));

    let mut connection1 = mock_connection::MockConnection::new(server.clone());
    let mut connection2 = mock_connection::MockConnection::new(server.clone());

    server.borrow_mut().connect(Box::new(&mut connection1));
    server.borrow_mut().connect(Box::new(&mut connection2));

    let mut client1 = Client::with_connection(&connection1).wait().unwrap();
    let mut client2 = Client::with_connection(&connection2).wait().unwrap();

    assert_eq!(client1.current_content().unwrap(), Default::default());
    assert_eq!(client2.current_content().unwrap(), Default::default());

    client1.push_operation(Operation::Op(
        vec![
            Cursor(Position {
                row: 0,
                col: "こんに".len(),
            }),
            Range(
                Position {
                    row: 0,
                    col: "こ".len(),
                },
                Position {
                    row: 1,
                    col: "世界".len(),
                },
            ),
        ],
        {
            let mut op = BaseOperation::new();
            op.insert("こんにちは".into()).insert("世界".into());
            op
        },
    ));
    {
        let (id, op) = client1.send_to_server().unwrap().wait().unwrap();
        client1.apply_patch(id, op).unwrap();
    }

    assert_eq!(
        client1.current_content().unwrap(),
        Target {
            base: vec!["こんにちは".into(), "世界".into()],
            selection: vec![
                Cursor(Position {
                    row: 0,
                    col: "こんに".len(),
                }),
                Range(
                    Position {
                        row: 0,
                        col: "こ".len(),
                    },
                    Position {
                        row: 1,
                        col: "世界".len(),
                    },
                ),
            ],
        }
    );
    assert_eq!(client2.current_content().unwrap(), Default::default());

    client2.push_operation(Operation::Op(
        vec![
            Cursor(Position {
                row: 0,
                col: "!".len(),
            }),
        ],
        {
            let mut op = BaseOperation::new();
            op.insert("!".into());
            op
        },
    ));
    {
        let (id, op) = client2.send_to_server().unwrap().wait().unwrap();
        client2.apply_patch(id, op).unwrap();
    }

    assert_eq!(
        client1.current_content().unwrap(),
        Target {
            base: vec!["こんにちは".into(), "世界".into()],
            selection: vec![
                Cursor(Position {
                    row: 0,
                    col: "こんに".len(),
                }),
                Range(
                    Position {
                        row: 0,
                        col: "こ".len(),
                    },
                    Position {
                        row: 1,
                        col: "世界".len(),
                    },
                ),
            ],
        }
    );
    assert_eq!(
        client2.current_content().unwrap(),
        Target {
            base: vec!["!".into(), "こんにちは".into(), "世界".into()],
            selection: vec![
                Cursor(Position {
                    row: 0,
                    col: "!".len(),
                }),
            ],
        }
    );

    {
        let target = client1.current_content().unwrap();
        client1.push_operation(target.operate({
            let mut op = BaseOperation::new();
            op.delete(1).insert("さようなら".into()).retain(1);
            op
        }));
    }
    {
        let (id, op) = client1.send_to_server().unwrap().wait().unwrap();
        client1.apply_patch(id, op).unwrap();
    }

    assert_eq!(
        client1.current_content().unwrap(),
        Target {
            base: vec!["!".into(), "さようなら".into(), "世界".into()],
            selection: vec![
                Cursor(Position { row: 2, col: 0 }),
                Range(
                    Position { row: 2, col: 0 },
                    Position {
                        row: 2,
                        col: "世界".len(),
                    },
                ),
            ],
        }
    );
    assert_eq!(
        client2.current_content().unwrap(),
        Target {
            base: vec!["!".into(), "こんにちは".into(), "世界".into()],
            selection: vec![
                Cursor(Position {
                    row: 0,
                    col: "!".len(),
                }),
            ],
        }
    );

    {
        let (id, op) = client2.send_get_patch().wait().unwrap();
        client2.apply_patch(id, op).unwrap();
    }

    assert_eq!(
        client1.current_content().unwrap(),
        Target {
            base: vec!["!".into(), "さようなら".into(), "世界".into()],
            selection: vec![
                Cursor(Position { row: 2, col: 0 }),
                Range(
                    Position { row: 2, col: 0 },
                    Position {
                        row: 2,
                        col: "世界".len(),
                    },
                ),
            ],
        }
    );
    assert_eq!(
        client2.current_content().unwrap(),
        Target {
            base: vec!["!".into(), "さようなら".into(), "世界".into()],
            selection: vec![
                Cursor(Position { row: 2, col: 0 }),
                Range(
                    Position { row: 2, col: 0 },
                    Position {
                        row: 2,
                        col: "世界".len(),
                    },
                ),
            ],
        }
    );
}
