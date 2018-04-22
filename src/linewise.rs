use super::Operation as OperationTrait;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LineOperation {
    Retain(usize),
    Insert(String),
    Modify(super::charwise::Operation),
    Delete(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Operation {
    operations: Vec<LineOperation>,
    source_len: usize,
    target_len: usize,
}

impl Operation {
    pub fn new() -> Self {
        Operation {
            operations: vec![],
            source_len: 0,
            target_len: 0,
        }
    }

    pub fn source_len(&self) -> usize {
        self.source_len
    }

    pub fn target_len(&self) -> usize {
        self.target_len
    }

    pub fn add(&mut self, op: LineOperation) {
        use self::LineOperation::*;
        match op {
            Retain(len) => {
                self.source_len += len;
                self.target_len += len;
                if let Some(&mut Retain(ref mut l)) = self.operations.last_mut() {
                    *l += len;
                    return;
                }
                self.operations.push(Retain(len));
            }
            Delete(len) => {
                self.source_len += len;
                if let Some(&mut Delete(ref mut l)) = self.operations.last_mut() {
                    *l += len;
                    return;
                }
                self.operations.push(Delete(len));
            }
            Modify(op) => {
                self.source_len += 1;
                self.target_len += 1;
                self.operations.push(Modify(op));
            }
            Insert(s) => {
                self.target_len += 1;
                self.operations.push(Insert(s));
            }
        }
    }

    pub fn retain(&mut self, len: usize) -> &mut Self {
        if len > 0 {
            self.add(LineOperation::Retain(len));
        }
        self
    }

    pub fn insert(&mut self, s: String) -> &mut Self {
        self.add(LineOperation::Insert(s));
        self
    }

    pub fn delete(&mut self, len: usize) -> &mut Self {
        if len > 0 {
            self.add(LineOperation::Delete(len));
        }
        self
    }

    pub fn modify(&mut self, op: super::charwise::Operation) -> &mut Self {
        self.add(LineOperation::Modify(op));
        self
    }
}

// apply operation to lines
pub fn apply(mut original: &[String], operation: &Operation) -> Vec<String> {
    assert_eq!(original.len(), operation.source_len);

    let mut ret = Vec::with_capacity(operation.target_len);

    for op in operation.operations.iter() {
        use self::LineOperation::*;

        match *op {
            Retain(len) => {
                for i in 0..len {
                    ret.push(original[i].to_string());
                }
                original = &original[len..];
            }
            Delete(len) => {
                original = &original[len..];
            }
            Insert(ref s) => {
                ret.push(s.to_string());
            }
            Modify(ref op) => {
                ret.push(op.apply(&original[0]));
                original = &original[1..];
            }
        }
    }

    ret
}

// compose two operations
// compose must satisfy apply(apply(s, a), b) == apply(s, compose(a, b))
pub fn compose(first: Operation, second: Operation) -> Operation {
    assert_eq!(first.target_len, second.source_len, "the target length of first operation {:?} and the source length of second operation {:?} must match", first, second);

    let mut ret = Operation::new();

    let mut first = first.operations.into_iter();
    let mut second = second.operations.into_iter();

    let mut head_first = first.next();
    let mut head_second = second.next();

    loop {
        use self::LineOperation::*;

        match (head_first, head_second) {
            (None, None) => break ret,
            (None, Some(value)) => {
                head_first = None;
                head_second = second.next();
                ret.add(value);
            },
            (Some(value), None) => {
                head_first = first.next();
                head_second = None;
                ret.add(value);
            },
            (Some(Delete(len)), s) => {
                head_first = first.next();
                head_second = s;
                ret.delete(len);
            },
            (f, Some(Insert(s))) => {
                head_first = f;
                head_second = second.next();
                ret.insert(s);
            },
            (Some(Retain(len_first)), Some(Retain(len_second))) => {
                if len_first < len_second {
                    head_first = first.next();
                    head_second = Some(Retain(len_second - len_first));
                    ret.retain(len_first);
                } else if len_first == len_second {
                    head_first = first.next();
                    head_second = second.next();
                    ret.retain(len_first);
                } else {
                    head_first = Some(Retain(len_first - len_second));
                    head_second = second.next();
                    ret.retain(len_second);
                }
            },
            (Some(Retain(len_first)), Some(Delete(len_second))) => {
                if len_first < len_second {
                    head_first = first.next();
                    head_second = Some(Delete(len_second - len_first));
                    ret.delete(len_first);
                } else if len_first == len_second {
                    head_first = first.next();
                    head_second = second.next();
                    ret.delete(len_first);
                } else {
                    head_first = Some(Retain(len_first - len_second));
                    head_second = second.next();
                    ret.delete(len_second);
                }
            },
            (Some(Retain(len)), Some(Modify(op))) => {
                if len == 0 {
                    unreachable!("length cannot be zero");
                } else if len == 1 {
                    head_first = first.next();
                } else {
                    head_first = Some(Retain(len - 1));
                }
                head_second = second.next();
                ret.modify(op);
            }
            (Some(Insert(_)), Some(Delete(len))) => {
                if len == 0 {
                    unreachable!("length cannot be zero");
                } else if len == 1 {
                    head_second = second.next();
                } else {
                    head_second = Some(Delete(len - 1));
                }
                head_first = first.next();
            },
            (Some(Insert(s)), Some(Retain(len))) => {
                if len == 0 {
                    unreachable!("length cannot be zero");
                } else if len == 1 {
                    head_second = second.next();
                } else {
                    head_second = Some(Retain(len - 1));
                }
                head_first = first.next();
                ret.insert(s);
            },
            (Some(Insert(s)), Some(Modify(op))) => {
                head_first = first.next();
                head_second = second.next();
                ret.insert(op.apply(&s));
            },
            (Some(Modify(op)), Some(Retain(len))) => {
                if len == 0 {
                    unreachable!("length cannot be zero");
                } else if len == 1 {
                    head_second = second.next();
                } else {
                    head_second = Some(Retain(len - 1));
                }
                head_first = first.next();
                ret.modify(op);
            },
            (Some(Modify(_)), Some(Delete(len))) => {
                if len == 0 {
                    unreachable!("length cannot be zero");
                } else if len == 1 {
                    head_second = second.next();
                } else {
                    head_second = Some(Delete(len - 1));
                }
                head_first = first.next();
                ret.delete(1);
            },
            (Some(Modify(lhs)), Some(Modify(rhs))) => {
                head_first = first.next();
                head_second = second.next();
                ret.modify(lhs.compose(rhs));
            }
        }
    }
}

// transforms two operations so that composed operations will converge
// let (left', right') = transform(left, right), these satisfies the condition
// apply(s, compose(left, right')) == apply(s, compose(right, left'))
pub fn transform(left: Operation, right: Operation) -> (Operation, Operation) {
    assert_eq!(left.source_len, right.source_len, "the source of both operation must match. left = {:?}, right = {:?}", left, right);

    let mut ret_left = Operation::new();
    let mut ret_right = Operation::new();

    let mut left = left.operations.into_iter();
    let mut right = right.operations.into_iter();

    let mut head_left = left.next();
    let mut head_right = right.next();

    loop {
        use self::LineOperation::*;

        match (head_left, head_right) {
            (None, None) => break (ret_left, ret_right),
            (Some(Insert(s)), value) => {
                ret_right.retain(1);
                ret_left.insert(s);
                head_left = left.next();
                head_right = value;
            },
            (value, Some(Insert(s))) => {
                ret_left.retain(1);
                ret_right.insert(s);
                head_left = value;
                head_right = right.next();
            },
            (None, _) => unreachable!("left is too short"),
            (_, None) => unreachable!("right is too short"),
            (Some(Retain(left_len)), Some(Retain(right_len))) => {
                let len;
                if left_len < right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = Some(Retain(right_len - left_len));
                } else if left_len == right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = right.next();
                } else {
                    len = right_len;
                    head_left = Some(Retain(left_len - right_len));
                    head_right = right.next();
                }
                ret_left.retain(len);
                ret_right.retain(len);
            },
            (Some(Delete(left_len)), Some(Delete(right_len))) => {
                if left_len < right_len {
                    head_left = left.next();
                    head_right = Some(Delete(right_len - left_len));
                } else if left_len == right_len {
                    head_left = left.next();
                    head_right = right.next();
                } else {
                    head_left = Some(Delete(left_len - right_len));
                    head_right = right.next();
                }
            },
            (Some(Modify(left_op)), Some(Modify(right_op))) => {
                head_left = left.next();
                head_right = right.next();
                let (left_op, right_op) = left_op.transform(right_op);
                ret_left.modify(left_op);
                ret_right.modify(right_op);
            },
            (Some(Retain(left_len)), Some(Delete(right_len))) => {
                let len;
                if left_len < right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = Some(Delete(right_len - left_len));
                } else if left_len == right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = right.next();
                } else {
                    len = right_len;
                    head_left = Some(Retain(left_len - right_len));
                    head_right = right.next();
                }
                ret_right.delete(len);
            },
            (Some(Delete(left_len)), Some(Retain(right_len))) => {
                let len;
                if left_len < right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = Some(Retain(right_len - left_len));
                } else if left_len == right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = right.next();
                } else {
                    len = right_len;
                    head_left = Some(Delete(left_len - right_len));
                    head_right = right.next();
                }
                ret_left.delete(len);
            },
            (Some(Modify(op)), Some(Retain(len))) => {
                if len == 0 {
                    unreachable!("length cannot be zero");
                } else if len == 1 {
                    head_right = right.next();
                } else {
                    head_right = Some(Retain(len - 1));
                }
                head_left = left.next();
                ret_left.modify(op);
                ret_right.retain(1);
            },
            (Some(Retain(len)), Some(Modify(op))) => {
                if len == 0 {
                    unreachable!("length cannot be zero");
                } else if len == 1 {
                    head_left = left.next();
                } else {
                    head_left = Some(Retain(len - 1));
                }
                head_right = right.next();
                ret_left.retain(1);
                ret_right.modify(op);
            },
            (Some(Modify(_)), Some(Delete(len))) => {
                if len == 0 {
                    unreachable!("length cannot be zero");
                } else if len == 1 {
                    head_right = right.next();
                } else {
                    head_right = Some(Delete(len - 1));
                }
                head_left = left.next();
                ret_right.delete(1);
            },
            (Some(Delete(len)), Some(Modify(_))) => {
                if len == 0 {
                    unreachable!("length cannot be zero");
                } else if len == 1 {
                    head_left = left.next();
                } else {
                    head_left = Some(Delete(len - 1));
                } 
                head_right = right.next();
                ret_left.delete(1);
            },
        }
    }
}

