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
    use rand::Rng;
    use rand::distributions::{Range, Sample};

    let mut op_type = Range::new(0, 3);
    let mut op_len = Range::new(0, 10);

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
    use rand::distributions::{Range, Sample};

    for _ in 0..100 {
        let mut rng = rand::thread_rng();
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