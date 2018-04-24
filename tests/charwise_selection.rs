extern crate ot;
use ot::selection::charwise::*;
use ot::charwise::Operation as BaseOperation;
use ot::Operation as OperationTrait;

mod util;

extern crate rand;

#[test]
fn test_apply() {
    use ot::selection::charwise::Selection::*;

    let target = Target {
        base: "こんにちは 世界".into(),
        selection: vec![
            Range("こんにちは".len(), "こんにちは ".len()),
            Cursor("こんにちは 世界".len()),
        ],
    };
    let op = Operation::Operate({
        let mut op = BaseOperation::new();
        op.retain("こんにちは".len())
            .insert("!".into())
            .retain(" ".len())
            .delete("世界".len())
            .insert("社会".into());
        op
    });

    assert_eq!(
        op.apply(&target),
        Target {
            base: "こんにちは! 社会".into(),
            selection: vec![
                // this behavior (extension of range) might be a specification bug
                // but hackmd seems to have the same behavior, so it's ok for now
                Range("こんにちは!".len(), "こんにちは! 社会".len()),
                Cursor("こんにちは! 社会".len()),
            ],
        }
    );
}

#[test]
fn test_compose() {
    use ot::selection::charwise::Selection::*;

    let target = Target {
        base: "こんにちは 世界".into(),
        selection: vec![],
    };
    let first = Operation::Both(
        vec![
            Range("こんにちは!".len(), "こんにちは! ".len()),
            Cursor("こんにちは! 世界".len()),
        ],
        {
            let mut op = BaseOperation::new();
            op.retain("こんにちは".len())
                .insert("!".into())
                .retain(" ".len())
                .delete("世界".len())
                .insert("社会".into());
            op
        },
    );
    let second = Operation::Operate({
        let mut op = BaseOperation::new();
        op.delete("こんにちは".len())
            .insert("さようなら".into())
            .retain("! 社会".len());
        op
    });

    assert_eq!(
        second.apply(&first.apply(&target)),
        first.clone().compose(second.clone()).apply(&target)
    );
    assert_eq!(
        second.apply(&first.apply(&target)),
        Target {
            base: "さようなら! 社会".into(),
            selection: vec![
                Range("さようなら!".len(), "さようなら! ".len()),
                Cursor("さようなら! 社会".len()),
            ],
        },
    );
    assert_eq!(
        first.compose(second).apply(&target),
        Target {
            base: "さようなら! 社会".into(),
            selection: vec![
                Range("さようなら!".len(), "さようなら! ".len()),
                Cursor("さようなら! 社会".len()),
            ],
        },
    );
}

#[test]
fn test_transform() {
    use ot::selection::charwise::Selection::*;

    let target = Target {
        base: "こんにちは 世界".into(),
        selection: vec![],
    };
    let left = Operation::Both(
        vec![
            Range("こんにちは!".len(), "こんにちは! ".len()),
            Cursor("こんにちは! 世界".len()),
        ],
        {
            let mut op = BaseOperation::new();
            op.retain("こんにちは".len())
                .insert("!".into())
                .retain(" ".len())
                .delete("世界".len())
                .insert("社会".into());
            op
        },
    );
    let right = Operation::Both(vec![Cursor("こ".len())], {
        let mut op = BaseOperation::new();
        op.delete("こんにちは".len())
            .insert("さようなら".into())
            .retain(" 世界".len());
        op
    });

    let (left_, right_) = left.clone().transform(right.clone());
    let composed_left = left.compose(right_);
    let composed_right = right.compose(left_);

    assert_eq!(composed_left.apply(&target), composed_right.apply(&target));
    assert_eq!(
        composed_left.apply(&target),
        Target {
            base: "!さようなら 社会".into(),
            selection: vec![
                Range("!さようなら".len(), "!さようなら ".len()),
                Cursor("!さようなら 社会".len()),
            ],
        }
    );
    assert_eq!(
        composed_left.apply(&target),
        Target {
            base: "!さようなら 社会".into(),
            selection: vec![
                Range("!さようなら".len(), "!さようなら ".len()),
                Cursor("!さようなら 社会".len()),
            ],
        }
    );
}

use rand::distributions::{Range, Sample};
fn random_selection<R: rand::Rng>(rng: &mut R, num_selection: usize, len: usize) -> Vec<Selection> {
    let mut range = Range::new(0, len + 1);
    (0..rng.gen_range(0, num_selection))
        .map(|_| {
            let start = range.sample(rng);
            let end = range.sample(rng);
            if start == end {
                Selection::Cursor(start)
            } else {
                Selection::Range(start.min(end), start.max(end))
            }
        })
        .collect()
}

fn random_target<R: rand::Rng>(rng: &mut R, num_selection: usize, len: usize) -> Target {
    let base = util::charwise::random_string(rng, len);
    let selection = random_selection(rng, num_selection, len);

    Target { base, selection }
}

fn random_operation<R: rand::Rng>(rng: &mut R, num_selection: usize, target: &Target) -> Operation {
    use Operation::*;

    let len = target.base.len();
    match rng.gen_range(0, 4) {
        0 => Nop,
        1 => Select(random_selection(rng, num_selection, len)),
        2 => Operate(util::charwise::random_operation(rng, &target.base)),
        3 => {
            let op = util::charwise::random_operation(rng, &target.base);
            let selection = random_selection(rng, num_selection, op.target_len());
            Both(selection, op)
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_random_operation() {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let len = rng.gen_range(32, 100);
    let selection_num = rng.gen_range(1, 30);
    let target = random_target(&mut rng, selection_num, len);

    let operation = random_operation(&mut rng, selection_num, &target);

    operation.apply(&target);
}

#[test]
fn fuzz_test_compose() {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let len = rng.gen_range(32, 100);
        let selection_num = rng.gen_range(1, 30);
        let target = random_target(&mut rng, selection_num, len);

        let first = random_operation(&mut rng, selection_num, &target);
        let applied = first.apply(&target);

        let second = random_operation(&mut rng, selection_num, &applied);

        let double_applied = second.apply(&applied);
        let compose_applied = first.compose(second).apply(&target);

        assert_eq!(double_applied, compose_applied);
    }
}

#[test]
fn fuzz_test_transform() {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let len = rng.gen_range(32, 100);
        let selection_num = rng.gen_range(1, 30);
        let target = random_target(&mut rng, selection_num, len);

        let left = random_operation(&mut rng, selection_num, &target);
        let right = random_operation(&mut rng, selection_num, &target);

        let (left_, right_) = left.clone().transform(right.clone());

        let left = left.compose(right_);
        let right = right.compose(left_);

        assert_eq!(left.apply(&target), right.apply(&target));
    }
}
