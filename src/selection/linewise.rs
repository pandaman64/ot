use super::super::Operation as OperationTrait;
use super::super::linewise::Operation as BaseOperation;

use std::default::Default;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Cursor(Position),
    Range(Position, Position),
}

impl Selection {
    fn transform_index(value: &mut Position, op: &BaseOperation) {
        use linewise::LineOperation::*;

        // index in row
        let mut idx = 0;
        for op in op.operations.iter() {
            println!("idx = {}, op = {:?}", idx, op);
            match *op {
                Retain(len) => {
                    idx += len;
                }
                Insert(_) => {
                    if idx <= value.row {
                        value.row += 1;
                    }
                    idx += 1;
                }
                Modify(ref op) => {
                    if idx == value.row {
                        super::charwise::Selection::transform_index(&mut value.col, op);
                    }
                    idx += 1;
                }
                Delete(len) => {
                    if idx < value.row {
                        value.row -= len.min(value.row - idx);
                    } else if idx == value.row {
                        value.col = 0;
                    }
                }
            }
        }
    }

    fn transform(mut self, op: &BaseOperation) -> Option<Self> {
        use self::Selection::*;

        match self {
            Cursor(ref mut pos) => Self::transform_index(pos, op),
            Range(ref mut start, ref mut end) => {
                Self::transform_index(start, op);
                Self::transform_index(end, op);

                // TODO: verify this
                if *start == *end {
                    return None;
                }
            }
        }

        Some(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Target<UserId: Clone + Eq + Hash> {
    pub base: <BaseOperation as OperationTrait>::Target,
    pub selection: HashMap<UserId, Vec<Selection>>,
}

impl<UserId: Clone + Eq + Hash> Default for Target<UserId> {
    fn default() -> Self {
        Target {
            base: <BaseOperation as OperationTrait>::Target::default(),
            selection: HashMap::new(),
        }
    }
}

impl<UserId: Clone + Eq + Hash> Target<UserId> {
    pub fn operate(&self, op: BaseOperation) -> Operation<UserId> {
        let selection = self.selection
            .iter()
            .map(|(id, s)| {
                (
                    id.clone(),
                    s.iter().cloned().filter_map(|s| s.transform(&op)).collect(),
                )
            })
            .collect();
        Operation::Op(selection, op)
    }

    pub fn select(&self, s: HashMap<UserId, Vec<Selection>>) -> Operation<UserId> {
        Operation::Op(s, {
            let mut op = BaseOperation::new();
            op.retain(self.base.len());
            op
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Operation<UserId: Clone + Eq + Hash> {
    Nop,
    Op(HashMap<UserId, Vec<Selection>>, BaseOperation),
}

impl<UserId: Clone + Eq + Hash> Operation<UserId> {
    pub fn with_content(op: BaseOperation) -> Self {
        Operation::Op(HashMap::new(), op)
    }

    pub fn operate(&self, op: BaseOperation) -> Self {
        use self::Operation::*;

        match *self {
            Nop => Op(HashMap::new(), op),
            Op(ref s, _) => {
                let s = s.iter()
                    .map(|(ref k, ref v)| {
                        let v = v.iter().cloned().filter_map(|s| s.transform(&op)).collect();
                        ((*k).clone(), v)
                    })
                    .collect();
                Op(s, op)
            }
        }
    }
}

impl<UserId: Clone + Eq + Hash> Default for Operation<UserId> {
    fn default() -> Self {
        Operation::Nop
    }
}

impl<UserId: Clone + Eq + Hash> OperationTrait for Operation<UserId> {
    type Target = Target<UserId>;

    fn nop(_: &Self::Target) -> Self {
        Operation::Nop
    }

    fn apply(&self, target: &Self::Target) -> Self::Target {
        use self::Operation::*;

        match *self {
            Nop => target.clone(),
            Op(ref s, ref op) => {
                let base = op.apply(&target.base);
                let selection = s.clone();

                Target { base, selection }
            }
        }
    }

    fn compose(self, other: Self) -> Self {
        use self::Operation::*;

        match (self, other) {
            (Nop, other) => other,
            (this, Nop) => this,
            (Op(_, lhs), Op(s, rhs)) => Op(s, lhs.compose(rhs)),
        }
    }

    // when each operation contains Select, tie break by adopting self's
    fn transform(self, other: Self) -> (Self, Self) {
        use self::Operation::*;

        match (self, other) {
            (Nop, other) => (Nop, other),
            (this, Nop) => (this, Nop),
            (Op(slhs, lhs), Op(srhs, rhs)) => {
                let (lhs_, rhs_) = lhs.transform(rhs);
                let selection: HashMap<UserId, Vec<Selection>> = {
                    let slhs = slhs.into_iter().map(|(id, s)| {
                        (
                            id,
                            s.into_iter().filter_map(|s| s.transform(&rhs_)).collect(),
                        )
                    });
                    let srhs = srhs.into_iter().map(|(id, s)| {
                        (
                            id,
                            s.into_iter().filter_map(|s| s.transform(&lhs_)).collect(),
                        )
                    });
                    srhs.chain(slhs).collect()
                };
                (Op(selection.clone(), lhs_), Op(selection, rhs_))
            }
        }
    }
}
