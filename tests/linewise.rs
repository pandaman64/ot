extern crate ot;
use ot::linewise::*;

mod charwise_util;

extern crate rand;

#[test]
fn test_apply() {
    let original = ["こんにちは", "世界"];
    let op = {
        let mut op = Operation::new();
        op.retain(1)
            .insert("!".into())
            .modify({
                let mut op = ot::charwise::Operation::new();
                op.delete("世界".len());
                op.insert("社会".into());
                op
            });
        op
    };

    assert_eq!(apply(&original, &op), ["こんにちは", "!", "社会"]);
}

#[test]
fn test_compose() {
    let original = ["こんにちは",  "世界"];
    let first = {
        let mut op = Operation::new();
        op.retain(1)
            .insert("!".into())
            .modify({
                let mut op = ot::charwise::Operation::new();
                op.delete("世界".len());
                op.insert("社会".into());
                op
            });
        op
    };
    let second = {
        let mut op = Operation::new();
        op.delete(1)
            .insert("さようなら".into())
            .retain(2);
        op
    };

    use std::borrow::Borrow;
    assert_eq!(
        apply(&apply(&original, &first).iter().map(Borrow::borrow).collect::<Vec<_>>(), &second),
        apply(&original, &compose(first.clone(), second.clone()))
    );
    assert_eq!(
        apply(&apply(&original, &first).iter().map(Borrow::borrow).collect::<Vec<_>>(), &second),
        ["さようなら", "!", "社会"]
    );
    assert_eq!(
        apply(&original, &compose(first, second)),
        ["さようなら", "!", "社会"]
    );
}

#[test]
fn test_transform() {
    let original = ["こんにちは", "世界"];
    let left = {
        let mut op = Operation::new();
        op.retain(1)
            .insert("!".into())
            .modify({
                let mut op = ot::charwise::Operation::new();
                op.delete("世界".len());
                op.insert("社会".into());
                op
            });
        op
    };
    let right = {
        let mut op = Operation::new();
        op.delete(1)
            .insert("さようなら".into())
            .retain(1);
        op
    };

    let (left_, right_) = transform(left.clone(), right.clone());
    let composed_left = compose(left, right_);
    let composed_right = compose(right, left_);

    assert_eq!(
        apply(&original, &composed_left),
        apply(&original, &composed_right)
    );
    assert_eq!(apply(&original, &composed_left), ["!", "さようなら", "社会"]);
    assert_eq!(apply(&original, &composed_right), ["!", "さようなら", "社会"]);
}

fn random_lines<R: rand::Rng>(rng: &mut R, max_line_len: usize, line_num: usize) -> Vec<String> {
    use rand::distributions::{Range, Sample};

    let mut len = Range::new(0, max_line_len + 1);
    (0..line_num).map(|_| {
        let len = len.sample(rng);
        charwise_util::random_string(rng, len)
    }).collect()
}

fn random_operation<R: rand::Rng>(rng: &mut R, original: &[&str]) -> Operation {
    use rand::distributions::{Range, Sample};

    let mut op_type = Range::new(0, 4);
    let mut ret = Operation::new();

    let mut idx = 0;

    while idx < original.len() {
        let mut op_len = Range::new(1, original.len() - idx + 1);
        match op_type.sample(rng) {
            // Retain
            0 => {
                let len = op_len.sample(rng);
                ret.retain(len);
                idx += len;
            }
            // Insert
            1 => {
                let len = rng.gen_range(0, 10);
                ret.insert(charwise_util::random_string(rng, len));
            }
            // Modify
            2 => {
                ret.modify(charwise_util::random_operation(rng, original[idx]));
                idx += 1;
            }
            // Delete
            3 => {
                let len = op_len.sample(rng);
                ret.delete(len);
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
    use std::borrow::Borrow;

    let mut rng = rand::thread_rng();
    let original_len = rng.gen_range(32, 100);
    let max_line_len = 30;
    let original = random_lines(&mut rng, max_line_len, original_len);

    let operation = random_operation(&mut rng, &original.iter().map(Borrow::borrow).collect::<Vec<_>>());

    assert_eq!(operation.source_len(), original.len());
}

#[test]
fn fuzz_test_compose() {
    use rand::Rng;
    use std::borrow::Borrow;

    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let original_len = rng.gen_range(32, 100);
        let max_line_len = 30;
        let original = random_lines(&mut rng, max_line_len, original_len);
        let original = original.iter().map(Borrow::borrow).collect::<Vec<_>>();

        let first = random_operation(&mut rng, &original);
        let applied = apply(&original, &first);
        let applied = applied.iter().map(Borrow::borrow).collect::<Vec<_>>();

        let second = random_operation(&mut rng, &applied);

        let double_applied = apply(&applied, &second);
        println!("-------------------\nfirst = {:?}\nsecond = {:?}\ncomposed = {:?}", first, second, compose(first.clone(), second.clone()));
        let compose_applied = apply(&original, &compose(first, second));

        assert_eq!(double_applied, compose_applied);
    }
}

#[test]
fn fuzz_test_transform() {
    use rand::Rng;
    use std::borrow::Borrow;

    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let original_len = rng.gen_range(32, 100);
        let max_line_len = 30;
        let original = random_lines(&mut rng, max_line_len, original_len);
        let original = original.iter().map(Borrow::borrow).collect::<Vec<_>>();

        let left = random_operation(&mut rng, &original);
        let right = random_operation(&mut rng, &original);

        let (left_, right_) = transform(left.clone(), right.clone());

        let left = compose(left, right_);
        let right = compose(right, left_);

        assert_eq!(apply(&original, &left), apply(&original, &right));
    }
}
/*
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
        client1.apply_patch(id, op).unwrap();
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
*/
