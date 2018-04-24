extern crate ot;
use ot::linewise::*;
use ot::Operation as OperationTrait;

mod util;
use util::linewise::*;

extern crate rand;

#[test]
fn test_apply() {
    let original = vec!["こんにちは".into(), "世界".into()];
    let op = {
        let mut op = Operation::new();
        op.retain(1).insert("!".into()).modify({
            let mut op = ot::charwise::Operation::new();
            op.delete("世界".len());
            op.insert("社会".into());
            op
        });
        op
    };

    assert_eq!(op.apply(&original), ["こんにちは", "!", "社会"]);
}

#[test]
fn test_compose() {
    let original = vec!["こんにちは".into(), "世界".into()];
    let first = {
        let mut op = Operation::new();
        op.retain(1).insert("!".into()).modify({
            let mut op = ot::charwise::Operation::new();
            op.delete("世界".len());
            op.insert("社会".into());
            op
        });
        op
    };
    let second = {
        let mut op = Operation::new();
        op.delete(1).insert("さようなら".into()).retain(2);
        op
    };

    assert_eq!(
        second.apply(&first.apply(&original)),
        first.clone().compose(second.clone()).apply(&original)
    );
    assert_eq!(
        second.apply(&first.apply(&original)),
        ["さようなら", "!", "社会"]
    );
    assert_eq!(
        first.compose(second).apply(&original),
        ["さようなら", "!", "社会"]
    );
}

#[test]
fn test_transform() {
    let original = vec!["こんにちは".into(), "世界".into()];
    let left = {
        let mut op = Operation::new();
        op.retain(1).insert("!".into()).modify({
            let mut op = ot::charwise::Operation::new();
            op.delete("世界".len());
            op.insert("社会".into());
            op
        });
        op
    };
    let right = {
        let mut op = Operation::new();
        op.delete(1).insert("さようなら".into()).retain(1);
        op
    };

    let (left_, right_) = left.clone().transform(right.clone());
    let composed_left = left.compose(right_);
    let composed_right = right.compose(left_);

    assert_eq!(
        composed_left.apply(&original),
        composed_right.apply(&original)
    );
    assert_eq!(
        composed_left.apply(&original),
        ["!", "さようなら", "社会"]
    );
    assert_eq!(
        composed_right.apply(&original),
        ["!", "さようなら", "社会"]
    );
}

#[test]
fn test_random_operation() {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let original_len = rng.gen_range(32, 100);
    let max_line_len = 30;
    let original = random_lines(&mut rng, max_line_len, original_len);

    let operation = random_operation(&mut rng, &original);

    assert_eq!(operation.source_len(), original.len());
}

#[test]
fn fuzz_test_compose() {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let original_len = rng.gen_range(32, 100);
        let max_line_len = 30;
        let original = random_lines(&mut rng, max_line_len, original_len);

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
        let max_line_len = 30;
        let original = random_lines(&mut rng, max_line_len, original_len);

        let left = random_operation(&mut rng, &original);
        let right = random_operation(&mut rng, &original);

        let (left_, right_) = left.clone().transform(right.clone());

        let left = left.compose(right_);
        let right = right.compose(left_);

        assert_eq!(left.apply(&original), right.apply(&original));
    }
}
