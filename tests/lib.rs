extern crate ot;
use ot::*;

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

