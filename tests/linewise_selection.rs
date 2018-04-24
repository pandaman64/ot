extern crate ot;
use ot::selection::linewise::*;
use ot::linewise::Operation as BaseOperation;
use ot::Operation as OperationTrait;

mod util;
use util::linewise_selection::*;

extern crate rand;

#[test]
fn test_apply() {
    use ot::selection::linewise::Position;
    use ot::selection::linewise::Selection::*;

    let target = Target {
        base: vec!["こんにちは".into(), "世界".into()],
        selection: vec![
            Range(Position {
                row: 0,
                col: "こんに".len()
            }, Position {
                row: 1,
                col: "世".len()
            }),
            Cursor(Position {
                row: 1,
                col: "世界".len()
            }),
        ],
    };
    let op = target.operate({
        let mut op = BaseOperation::new();
        op.retain(1)
            .insert("!".into())
            .modify({
                let mut op = ot::charwise::Operation::new();
                op.delete("世界".len());
                op.insert("社会".into());
                op
            });
        op
    });

    assert_eq!(op.apply(&target), Target {
        base: vec!["こんにちは".into(), "!".into(), "社会".into()],
        selection: vec![
            Range(Position {
                row: 0,
                col: "こんに".len()
            }, Position {
                row: 2,
                col: "社会".len()
            }),
            Cursor(Position {
                row: 2,
                col: "社会".len()
            }),
        ]
    });
}

#[test]
fn test_compose() {
    use ot::selection::linewise::Position;
    use ot::selection::linewise::Selection::*;

    let target = Target {
        base: vec!["こんにちは".into(), "世界".into()],
        selection: vec![
            Range(Position {
                row: 0,
                col: "こんに".len()
            }, Position {
                row: 1,
                col: "世".len()
            }),
            Cursor(Position {
                row: 1,
                col: "世界".len()
            }),
        ],
    };
    let first = target.operate({
        let mut op = BaseOperation::new();
        op.retain(1)
            .insert("!".into())
            .modify({
                let mut op = ot::charwise::Operation::new();
                op.delete("世界".len());
                op.insert("社会".into());
                op
            });
        op
    });
    let applied = first.apply(&target);
    let second = applied.operate({
        let mut op = BaseOperation::new();
        op.delete(1)
            .insert("さようなら".into())
            .retain(2);
        op
    });

    assert_eq!(
        second.apply(&first.apply(&target)),
        first.clone().compose(second.clone()).apply(&target)
    );
    assert_eq!(
        second.apply(&first.apply(&target)),
        Target {
            base: vec!["さようなら".into(), "!".into(), "社会".into()],
            selection: vec![
                Range(Position {
                    row: 1,
                    col: 0,
                }, Position {
                    row: 2,
                    col: "社会".len(),
                }),
                Cursor(Position {
                    row: 2,
                    col: "社会".len(),
                }),
            ],
        }
    );
    assert_eq!(
        first.compose(second).apply(&target),
        Target {
            base: vec!["さようなら".into(), "!".into(), "社会".into()],
            selection: vec![
                Range(Position {
                    row: 1,
                    col: 0,
                }, Position {
                    row: 2,
                    col: "社会".len(),
                }),
                Cursor(Position {
                    row: 2,
                    col: "社会".len(),
                }),
            ],
        }
    );
}

#[test]
fn test_transform() {
    use ot::selection::linewise::Position;
    use ot::selection::linewise::Selection::*;

    let target = Target {
        base: vec!["こんにちは".into(), "世界".into()],
        selection: vec![
            Range(Position {
                row: 0,
                col: "こんに".len()
            }, Position {
                row: 1,
                col: "世".len()
            }),
            Cursor(Position {
                row: 0,
                col: "こんにち".len()
            }),
        ],
    };
    let left = target.operate({
        let mut op = BaseOperation::new();
        op.retain(1)
            .insert("!".into())
            .modify({
                let mut op = ot::charwise::Operation::new();
                op.delete("世界".len());
                op.insert("社会".into());
                op
            });
        op
    });
    let right = Operation::Op(vec![
        Cursor(Position {
            row: 0,
            col: "こ".len(),
        }),
    ], {
        let mut op = BaseOperation::new();
        op.delete(1)
            .insert("さようなら".into())
            .retain(1);
        op
    });

    let (left_, right_) = left.clone().transform(right.clone());
    let composed_left = left.compose(right_);
    let composed_right = right.compose(left_);

    assert_eq!(
        composed_left.apply(&target),
        composed_right.apply(&target)
    );
    assert_eq!(composed_left.apply(&target), Target {
        base: vec![
            "!".into(),
            "さようなら".into(),
            "社会".into()
        ],
        selection: vec![
            Range(Position {
                row: 0,
                col: 0,
            }, Position {
                row: 2,
                col: "社会".len(),
            }),
            Cursor(Position {
                row: 0,
                col: 0,
            }),
        ], 
    });
    assert_eq!(composed_left.apply(&target), Target {
        base: vec![
            "!".into(),
            "さようなら".into(),
            "社会".into()
        ],
        selection: vec![
            Range(Position {
                row: 0,
                col: 0,
            }, Position {
                row: 2,
                col: "社会".len(),
            }),
            Cursor(Position {
                row: 0,
                col: 0,
            }),
        ], 
    });
}


#[test]
fn test_random_operation() {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let len = rng.gen_range(32, 100);
    let max_line_len = 30;
    let selection_num = rng.gen_range(1, 30);
    let target = random_target(&mut rng, selection_num,  max_line_len, len);

    let operation = random_operation(&mut rng, selection_num, &target);

    operation.apply(&target);
}

#[test]
fn fuzz_test_compose() {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let len = rng.gen_range(32, 100);
        let max_line_len = 30;
        let selection_num = rng.gen_range(1, 30);
        let target = random_target(&mut rng, selection_num, max_line_len, len);

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
        let max_line_len = 30;
        let selection_num = rng.gen_range(1, 30);
        let target = random_target(&mut rng, selection_num, max_line_len, len);

        let left = random_operation(&mut rng, selection_num, &target);
        let right = random_operation(&mut rng, selection_num, &target);

        let (left_, right_) = left.clone().transform(right.clone());

        let left = left.compose(right_);
        let right = right.compose(left_);

        assert_eq!(left.apply(&target), right.apply(&target));
    }
}

