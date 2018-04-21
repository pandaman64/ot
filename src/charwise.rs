// This source code is essentially a rewrite of https://github.com/hackmdio/hackmd/blob/master/lib/ot/text-operation.js

#[derive(Serialize, Deserialize, Debug, Clone)]
enum PrimitiveOperation {
    // skip n bytes of string
    Retain(usize),
    // insert a string
    Insert(String),
    // delete next n bytes
    Delete(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Operation {
    operations: Vec<PrimitiveOperation>,
    // the length of the original string, in bytes
    source_len: usize,
    // the length of the applied string, in bytes
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

    fn add(&mut self, op: PrimitiveOperation) {
        use self::PrimitiveOperation::*;
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
            Insert(s) => {
                self.target_len += s.len();
                if let Some(&mut Insert(ref mut ss)) = self.operations.last_mut() {
                    ss.push_str(&s);
                    return;
                }
                self.operations.push(Insert(s));
            }
            Delete(len) => {
                self.source_len += len;
                if let Some(&mut Delete(ref mut l)) = self.operations.last_mut() {
                    *l += len;
                    return;
                }
                self.operations.push(Delete(len));
            }
        }
    }

    // NOTE: len is in bytes
    pub fn retain(&mut self, len: usize) -> &mut Self {
        if len > 0 {
            self.add(PrimitiveOperation::Retain(len));
        }
        self
    }

    pub fn insert(&mut self, s: String) -> &mut Self {
        if s.len() > 0 {
            self.add(PrimitiveOperation::Insert(s));
        }
        self
    }

    // NOTE: len is in bytes
    pub fn delete(&mut self, len: usize) -> &mut Self {
        if len > 0 {
            self.add(PrimitiveOperation::Delete(len));
        }
        self
    }
}

// apply operation to string
pub fn apply(mut original: &str, operation: &Operation) -> String {
    let mut ret = String::with_capacity(operation.target_len);

    assert_eq!(original.len(), operation.source_len, "the length of string {} and the source length of operation {:?} must match", original, operation);

    for op in operation.operations.iter() {
        use self::PrimitiveOperation::*;
        match *op {
            Retain(len) => {
                ret.push_str(&original[0..len]);
                original = &original[len..];
            }
            Insert(ref s) => ret.push_str(s),
            Delete(len) => {
                original = &original[len..];
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
        use self::PrimitiveOperation::*;

        match (head_first, head_second) {
            (None, None) => break ret,
            (None, value) => {
                head_first = None;
                head_second = second.next();
                ret.add(value.unwrap());
            },
            (value, None) => {
                head_first = first.next();
                head_second = None;
                ret.add(value.unwrap());
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
            (Some(Insert(mut s)), Some(Delete(len))) => {
                if s.len() < len {
                    head_first = first.next();
                    head_second = Some(Delete(len - s.len()));
                } else if s.len() == len {
                    head_first = first.next();
                    head_second = second.next();
                } else {
                    head_first = Some(Insert(s.split_off(len)));
                    head_second = second.next();
                }
            },
            (Some(Insert(mut s)), Some(Retain(len))) => {
                if s.len() < len {
                    head_first = first.next();
                    head_second = Some(Retain(len - s.len()));
                } else if s.len() == len {
                    head_first = first.next();
                    head_second = second.next();
                } else {
                    head_first = Some(Insert(s.split_off(len)));
                    head_second = second.next();
                }
                ret.insert(s);
            },
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
        use self::PrimitiveOperation::*;

        match (head_left, head_right) {
            (None, None) => break (ret_left, ret_right),
            (Some(Insert(s)), value) => {
                ret_right.retain(s.len());
                ret_left.insert(s);
                head_left = left.next();
                head_right = value;
            },
            (value, Some(Insert(s))) => {
                ret_left.retain(s.len());
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
        }
    }
}
