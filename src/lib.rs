// This source code is essentially a rewrite of https://github.com/hackmdio/hackmd/blob/master/lib/ot/text-operation.js

pub mod util;
pub mod server;
pub mod client;

#[derive(Debug, Clone)]
enum PrimitiveOperation {
    // skip n bytes of string
    Retain(usize),
    // insert a string
    Insert(String),
    // delete next n bytes
    Delete(usize),
}

#[derive(Debug, Clone)]
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
        use PrimitiveOperation::*;
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
        self.add(PrimitiveOperation::Retain(len));
        self
    }

    pub fn insert(&mut self, s: String) -> &mut Self {
        self.add(PrimitiveOperation::Insert(s));
        self
    }

    // NOTE: len is in bytes
    pub fn delete(&mut self, len: usize) -> &mut Self {
        self.add(PrimitiveOperation::Delete(len));
        self
    }
}

// apply operation to string
pub fn apply(mut original: &str, operation: &Operation) -> String {
    let mut ret = String::with_capacity(operation.target_len);

    assert_eq!(original.len(), operation.source_len);

    for op in operation.operations.iter() {
        use PrimitiveOperation::*;
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
    assert_eq!(first.target_len, second.source_len);

    let mut ret = Operation::new();

    let mut first = first.operations.into_iter();
    let mut second = second.operations.into_iter();

    let mut head_first = first.next();
    let mut head_second = second.next();

    loop {
        use PrimitiveOperation::*;

        if head_first.is_none() {
            if head_second.is_none() {
                break ret;
            } else {
                ret.add(std::mem::replace(&mut head_second, second.next()).unwrap());
                continue;
            }
        } else if head_second.is_none() {
            ret.add(std::mem::replace(&mut head_first, first.next()).unwrap());
            continue;
        }

        if let Some(Delete(_)) = head_first {
            ret.add(std::mem::replace(&mut head_first, first.next()).unwrap());
            continue;
        }

        if let Some(Insert(_)) = head_second {
            ret.add(std::mem::replace(&mut head_second, second.next()).unwrap());
            continue;
        }

        if let Some(Retain(len_first)) = head_first {
            // if both heads are Retain, consume the shorter one and add it to the result
            // if both Retain has same length, consume both
            if let Some(Retain(len_second)) = head_second {
                if len_first < len_second {
                    ret.retain(len_first);
                    head_first = first.next();
                    head_second = Some(Retain(len_second - len_first));
                } else if len_first == len_second {
                    ret.retain(len_first);
                    head_first = first.next();
                    head_second = second.next();
                } else
                /* if len_first > len_second */
                {
                    ret.retain(len_second);
                    head_first = Some(Retain(len_first - len_second));
                    head_second = second.next();
                }
                continue;
            }

            // Retain/Delete case
            if let Some(Delete(len_second)) = head_second {
                if len_first < len_second {
                    ret.delete(len_first);
                    head_first = first.next();
                    head_second = Some(Delete(len_second - len_first));
                } else if len_first == len_second {
                    ret.delete(len_first);
                    head_first = first.next();
                    head_second = second.next();
                } else
                /* if len_first > len_second */
                {
                    ret.delete(len_second);
                    head_first = Some(Retain(len_first - len_second));
                    head_second = second.next();
                }
                continue;
            }
        }

        if let Some(Insert(_)) = head_first {
            // Insert/Delete
            if let Some(Delete(len)) = head_second {
                if let Some(Insert(mut s)) = head_first {
                    if s.len() < len {
                        head_first = first.next();
                        head_second = Some(Delete(len - s.len()));
                    } else if s.len() == len {
                        head_first = first.next();
                        head_second = second.next();
                    } else
                    /* if s.len() > len */
                    {
                        head_first = Some(Insert(s.split_off(len)));
                        head_second = second.next();
                    }
                }
                continue;
            }

            // Insert/Retain
            if let Some(Retain(len)) = head_second {
                if let Some(Insert(mut s)) = head_first {
                    if s.len() < len {
                        head_first = first.next();
                        head_second = Some(Retain(len - s.len()));
                        ret.insert(s);
                    } else if s.len() == len {
                        head_first = first.next();
                        head_second = second.next();
                        ret.insert(s);
                    } else
                    /* if s.len() > len */
                    {
                        let latter = s.split_off(len);
                        head_first = Some(Insert(latter));
                        head_second = second.next();
                        ret.insert(s);
                    }
                }
                continue;
            }
        }

        // because each branch ended with continue,
        // reaching here means we have missing case
        panic!(
            "missing case! head_first = {:?}, head_second = {:?}",
            head_first, head_second
        );
    }
}

// transforms two operations so that composed operations will converge
// let (left', right') = transform(left, right), these satisfies the condition
// apply(s, compose(left, right')) == apply(s, compose(right, left'))
pub fn transform(left: Operation, right: Operation) -> (Operation, Operation) {
    assert_eq!(left.source_len, right.source_len);

    let mut ret_left = Operation::new();
    let mut ret_right = Operation::new();

    let mut left = left.operations.into_iter();
    let mut right = right.operations.into_iter();

    let mut head_left = left.next();
    let mut head_right = right.next();

    loop {
        use PrimitiveOperation::*;

        if head_left.is_none() && head_right.is_none() {
            break (ret_left, ret_right);
        }

        // if either of the head of operations is Insert, add it to the other
        // if both of them are Insert, break a tie by adopting the left
        if let Some(Insert(s)) = head_left {
            ret_right.retain(s.len());
            ret_left.insert(s);
            head_left = left.next();
            continue;
        }

        if let Some(Insert(s)) = head_right {
            ret_left.retain(s.len());
            ret_right.insert(s);
            head_right = right.next();
            continue;
        }

        if head_left.is_none() {
            panic!("reaching here is a bug: left is too short");
        }

        if head_right.is_none() {
            panic!("reaching here is a bug: right is too short");
        }

        if let Some(Retain(left_len)) = head_left {
            if let Some(Retain(right_len)) = head_right {
                let len;
                if left_len < right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = Some(Retain(right_len - left_len));
                } else if left_len == right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = right.next();
                } else
                /* left_len > right_len */
                {
                    len = right_len;
                    head_left = Some(Retain(left_len - right_len));
                    head_right = right.next();
                }
                ret_right.retain(len);
                ret_left.retain(len);
                continue;
            }

            if let Some(Delete(right_len)) = head_right {
                let len;
                if left_len < right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = Some(Delete(right_len - left_len));
                } else if left_len == right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = right.next();
                } else
                /* left_len > right_len */
                {
                    len = right_len;
                    head_left = Some(Retain(left_len - right_len));
                    head_right = right.next();
                }
                ret_right.delete(len);
                continue;
            }
        }

        if let Some(Delete(left_len)) = head_left {
            if let Some(Retain(right_len)) = head_right {
                let len;
                if left_len < right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = Some(Retain(right_len - left_len));
                } else if left_len == right_len {
                    len = left_len;
                    head_left = left.next();
                    head_right = right.next();
                } else
                /* left_len > right_len */
                {
                    len = right_len;
                    head_left = Some(Delete(left_len - right_len));
                    head_right = right.next();
                }
                ret_left.delete(len);
                continue;
            }

            if let Some(Delete(right_len)) = head_right {
                if left_len < right_len {
                    head_left = left.next();
                    head_right = Some(Delete(right_len - left_len));
                } else if left_len == right_len {
                    head_left = left.next();
                    head_right = right.next();
                } else
                /* left_len > right_len */
                {
                    head_left = Some(Delete(left_len - right_len));
                    head_right = right.next();
                }
            }
            continue;
        }

        // because each branch ended with continue,
        // reaching here means we have missing case
        panic!(
            "missing case! head_left = {:?}, head_right = {:?}",
            head_left, head_right
        );
    }
}
