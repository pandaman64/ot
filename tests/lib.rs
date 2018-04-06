extern crate ot;
use ot::*;

extern crate rand;

#[test]
fn test_apply() {
    let original = "こんにちは 世界";
    let op = {
        let mut op = Operation::new();
        op.retain("こんにちは".len())
            .insert("!".into())
            .retain(" ".len())
            .delete("世界".len())
            .insert("社会".into());
        op
    };

    assert_eq!(apply(original, &op), "こんにちは! 社会");
}

#[test]
fn test_compose() {
    let original = "こんにちは 世界";
    let first = {
        let mut op = Operation::new();
        op.retain("こんにちは".len())
            .insert("!".into())
            .retain(" ".len())
            .delete("世界".len())
            .insert("社会".into());
        op
    };
    let second = {
        let mut op = Operation::new();
        op.delete("こんにちは".len())
            .insert("さようなら".into())
            .retain("! 社会".len());
        op
    };

    assert_eq!(
        apply(&apply(original, &first), &second),
        apply(original, &compose(first.clone(), second.clone()))
    );
    assert_eq!(
        apply(&apply(original, &first), &second),
        "さようなら! 社会"
    );
    assert_eq!(
        apply(original, &compose(first, second)),
        "さようなら! 社会"
    );
}

#[test]
fn test_transform() {
    let original = "こんにちは 世界";
    let left = {
        let mut op = Operation::new();
        op.retain("こんにちは".len())
            .insert("!".into())
            .retain(" ".len())
            .delete("世界".len())
            .insert("社会".into());
        op
    };
    let right = {
        let mut op = Operation::new();
        op.delete("こんにちは".len())
            .insert("さようなら".into())
            .retain(" 世界".len());
        op
    };

    let (left_, right_) = transform(left.clone(), right.clone());
    let composed_left = compose(left, right_);
    let composed_right = compose(right, left_);

    assert_eq!(
        apply(original, &composed_left),
        apply(original, &composed_right)
    );
    assert_eq!(apply(original, &composed_left), "!さようなら 社会");
    assert_eq!(apply(original, &composed_right), "!さようなら 社会");
}

fn random_string<R: rand::Rng>(rng: &mut R, len: usize) -> String {
    rng.gen_iter::<char>().take(len).collect()
}

fn random_operation<R: rand::Rng>(rng: &mut R, original: &str) -> Operation {
    use rand::distributions::{Range, Sample};

    let mut op_type = Range::new(0, 3);

    let mut ret = Operation::new();

    let chars = original.chars().collect::<Vec<_>>();

    let mut idx = 0;

    while idx < chars.len() {
        let mut op_len = Range::new(1, chars.len() - idx + 1);
        match op_type.sample(rng) {
            // Retain
            0 => {
                let len = op_len.sample(rng);
                let mut bytes = 0;
                for c in chars[idx..(idx + len)].iter() {
                    bytes += c.len_utf8();
                }
                ret.retain(bytes);
                idx += len;
            }
            // Insert
            1 => {
                let len = rng.gen_range(0, 10);
                ret.insert(random_string(rng, len));
            }
            // Delete
            2 => {
                let len = op_len.sample(rng);
                let mut bytes = 0;
                for c in chars[idx..(idx + len)].iter() {
                    bytes += c.len_utf8();
                }
                ret.delete(bytes);
                idx += len;
            }
            _ => unreachable!(),
        }
    }

    ret
}

#[test]
fn test_random_operation() {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let original_len = rng.gen_range(32, 100);
    let original = random_string(&mut rng, original_len);

    let operation = random_operation(&mut rng, &original);

    assert_eq!(operation.source_len(), original.len());
}

#[test]
fn fuzz_test_compose() {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let original_len = rng.gen_range(32, 100);
        let original = random_string(&mut rng, original_len);

        let first = random_operation(&mut rng, &original);
        let applied = apply(&original, &first);

        let second = random_operation(&mut rng, &applied);

        let double_applied = apply(&applied, &second);
        let compose_applied = apply(&original, &compose(first, second));

        assert_eq!(double_applied, compose_applied);
    }
}

#[test]
fn fuzz_test_transform() {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let original_len = rng.gen_range(32, 100);
        let original = random_string(&mut rng, original_len);

        let left = random_operation(&mut rng, &original);
        let right = random_operation(&mut rng, &original);

        let (left_, right_) = transform(left.clone(), right.clone());

        let left = compose(left, right_);
        let right = compose(right, left_);

        assert_eq!(apply(&original, &left), apply(&original, &right));
    }
}

#[macro_use]
extern crate failure;
extern crate futures;

#[test]
fn test_client_server() {
    use std::rc::Rc;
    use std::cell::RefCell;

    use self::futures::Future;

    use ot::*;
    use ot::util::*;
    use ot::server::*;
    use ot::client::*;

    struct MockConnection(Rc<RefCell<Server>>);

    impl<'a> server::Connection for &'a MockConnection {
        fn send_state(&mut self, _state: &State) {
        }
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
        client1.apply_response(id, op).unwrap();
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
        client2.apply_response(id, op).unwrap();
    }

    assert_eq!(client1.current_content().unwrap(), "こんにちは 世界");
    assert_eq!(client2.current_content().unwrap(), "!こんにちは 世界");

    client1.push_operation({
        let mut op = Operation::new();
        op.delete("こんにちは".len());
        op.insert("さようなら".into());
        op.retain(" 世界".len());
        op
    });
    {
        let (id, op) = client1.send_to_server().unwrap().wait().unwrap();
        client1.apply_response(id, op).unwrap();
    }

    assert_eq!(client1.current_content().unwrap(), "!さようなら 世界");
    assert_eq!(client2.current_content().unwrap(), "!こんにちは 世界");
    
    {
        let (latest_id, diff) = client2.send_get_patch().wait().unwrap();
        client2.apply_patch(latest_id, diff).unwrap();
    }

    assert_eq!(client1.current_content().unwrap(), "!さようなら 世界");
    assert_eq!(client2.current_content().unwrap(), "!さようなら 世界");
}

