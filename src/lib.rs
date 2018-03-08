// This source code is essentially a rewrite of https://github.com/hackmdio/hackmd/blob/master/lib/ot/text-operation.js

#[derive(Debug)]
enum PrimitiveOperation {
    // skip n bytes of string
    Retain(usize),
    // insert a string
    Insert(String),
    // delete next n bytes
    Delete(usize),
}

#[derive(Debug)]
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
            },
            Insert(s) => {
                self.target_len += s.len();
                if let Some(&mut Insert(ref mut ss)) = self.operations.last_mut() {
                    ss.push_str(&s);
                    return;
                }
                self.operations.push(Insert(s));
            },
            Delete(len) => {
                self.source_len += len;
                if let Some(&mut Delete(ref mut l)) = self.operations.last_mut() {
                    *l += len;
                    return;
                }
                self.operations.push(Delete(len));
            },
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
            },
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
                } else /* if len_first > len_second */ {
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
                } else /* if len_first > len_second */ {
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
                    } else /* if s.len() > len */ {
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
                    } else /* if s.len() > len */ {
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
        panic!("missing case! head_first = {:?}, head_second = {:?}", head_first, head_second);
    }
}

// transforms two operations so that composed operations will converge
// let (left', right') = transform(left, right), these satisfies the condition
// apply(s, compose(left, right')) == apply(s, compose(right, left'))
pub fn transform(left: &Operation, right: &Operation) -> (Operation, Operation) {
    unimplemented!()
}

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

    assert_eq!(apply(&apply(original, &first), &second), "さようなら! 社会");
    assert_eq!(apply(original, &compose(first, second)), "さようなら! 社会");
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

    let (left_, right_) = transform(&left, &right);

    assert_eq!(apply(original, &compose(left, right_)), apply(original, &compose(right, left_)));
}

