extern crate ot;
use ot::charwise::*;
use ot::Operation as OperationTrait;

mod util;
use util::charwise::*;

#[macro_use]
extern crate failure;
extern crate futures;
extern crate rand;

#[test]
fn test_apply() {
    let original = "こんにちは 世界".into();
    let op = {
        let mut op = Operation::new();
        op.retain("こんにちは".len())
            .insert("!".into())
            .retain(" ".len())
            .delete("世界".len())
            .insert("社会".into());
        op
    };

    assert_eq!(op.apply(&original), "こんにちは! 社会");
}

#[test]
fn test_compose() {
    let original = "こんにちは 世界".into();
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
        second.apply(&first.apply(&original)),
        first.clone().compose(second.clone()).apply(&original)
    );
    assert_eq!(
        second.apply(&first.apply(&original)),
        "さようなら! 社会"
    );
    assert_eq!(
        first.compose(second).apply(&original),
        "さようなら! 社会"
    );
}

#[test]
fn test_transform() {
    let original = "こんにちは 世界".into();
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

    let (left_, right_) = left.clone().transform(right.clone());
    let composed_left = left.compose(right_);
    let composed_right = right.compose(left_);

    assert_eq!(
        composed_left.apply(&original),
        composed_right.apply(&original)
    );
    assert_eq!(composed_left.apply(&original), "!さようなら 社会");
    assert_eq!(composed_right.apply(&original), "!さようなら 社会");
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
        let applied = first.apply(&original);

        let second = random_operation(&mut rng, &applied);

        let double_applied = second.apply(&applied);
        let compose_applied = first.compose(second).apply(&original);

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

        let (left_, right_) = left.clone().transform(right.clone());

        let left = left.compose(right_);
        let right = right.compose(left_);

        assert_eq!(left.apply(&original), right.apply(&original));
    }
}
