use super::super::Operation as OperationTrait;
use super::super::charwise::Operation as BaseOperation;

use std::default::Default;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Cursor(usize),
    Range(usize, usize),
}

impl Selection {
    fn transform_index(value: &mut usize, op: &BaseOperation)  {
        use charwise::PrimitiveOperation::*;

        let mut idx = 0;
        for op in op.operations.iter() {
            match *op {
                Retain(len) => {
                    idx += len;
                },
                Insert(ref s) => {
                    if idx <= *value {
                        *value += s.len();
                    }
                    idx += s.len();
                },
                Delete(len) => {
                    if idx <= *value {
                        *value -= len.min(*value - idx);
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
pub struct Target {
    pub base: <BaseOperation as OperationTrait>::Target,
    pub selection: Vec<Selection>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Operation {
    Nop,
    Select(Vec<Selection>),
    Operate(BaseOperation),
    Both(Vec<Selection>,BaseOperation),
}

impl Default for Operation {
    fn default() -> Self {
        Operation::Nop
    }
}

impl OperationTrait for Operation {
    type Target = Target;

    fn apply(&self, target: &Target) -> Target {
        use self::Operation::*;

        match *self {
            Nop => target.clone(),
            Select(ref s) => Target {
                base: target.base.clone(),
                selection: s.clone(),
            },
            Operate(ref op) => {
                let base = op.apply(&target.base);
                let selection = target.selection.iter().cloned().filter_map(|s| s.transform(op)).collect();

                Target {
                    base,
                    selection,
                }
            },
            Both(ref s, ref op) => {
                let base = op.apply(&target.base);
                let selection = s.clone();

                Target {
                    base,
                    selection,
                }
            },
        }
    }

    fn compose(self, other: Self) -> Self {
        use self::Operation::*;

        match (self, other) {
            (Nop, other) => other,
            (this, Nop) => this,
            (Select(_), Select(s)) => Select(s),
            (Select(s), Operate(o)) => {
                let selection = s.into_iter().filter_map(|s| s.transform(&o)).collect();
                Both(selection, o)
            },
            (Select(_), Both(s, o)) | (Operate(o), Select(s)) | (Both(_, o), Select(s)) => Both(s, o),
            (Operate(lhs), Operate(rhs)) => Operate(lhs.compose(rhs)),
            (Operate(lhs), Both(s, rhs)) | (Both(_, lhs), Both(s, rhs)) => {
                Both(s, lhs.compose(rhs))
            },
            (Both(s, lhs), Operate(rhs)) => {
                let s = s.into_iter().filter_map(|s| s.transform(&rhs)).collect();
                Both(s, lhs.compose(rhs))
            },
        }
    }

    // when each operation contains Select, tie break by adopting self's
    fn transform(self, other: Self) -> (Self, Self) {
        use self::Operation::*;

        match (self, other) {
            (Nop, other) => (Nop, other),
            (this, Nop) => (this, Nop),
            (Select(s), Select(_)) => (Select(s), Nop),
            (Select(s), Operate(o)) | (Select(s), Both(_, o)) => {
                let selection = s.into_iter().filter_map(|s| s.transform(&o)).collect();
                (Select(selection), Operate(o))
            },
            (Operate(o), Select(s)) => {
                let selection = s.into_iter().filter_map(|s| s.transform(&o)).collect();
                (Operate(o), Select(selection))
            },
            (Operate(lhs), Operate(rhs)) => {
                let (lhs_, rhs_) = lhs.transform(rhs);
                (Operate(lhs_), Operate(rhs_))
            }
            (Operate(lhs), Both(s, rhs)) => {
                let (lhs_, rhs_) = lhs.transform(rhs);
                let selection = s.into_iter().filter_map(|s| s.transform(&lhs_)).collect();
                (Operate(lhs_), Both(selection, rhs_))
            },
            (Both(s, o), Select(_)) => (Both(s, o), Nop),
            (Both(s, lhs), Operate(rhs)) | (Both(s, lhs), Both(_, rhs)) => {
                let (lhs_, rhs_) = lhs.transform(rhs);
                let selection = s.into_iter().filter_map(|s| s.transform(&rhs_)).collect::<Vec<_>>();
                (Both(selection.clone(), lhs_), Both(selection, rhs_))
            },
        }
    }
}

