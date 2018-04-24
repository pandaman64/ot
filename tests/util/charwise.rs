use ot::charwise::*;
use super::rand;

pub fn random_string<R: rand::Rng>(rng: &mut R, len: usize) -> String {
    rng.gen_iter::<char>().take(len).collect()
}

pub fn random_operation<R: rand::Rng>(rng: &mut R, original: &str) -> Operation {
    use rand::distributions::{Range, Sample};

    let mut op_type = Range::new(0, 3);

    let mut ret = Operation::new();

    let chars = original.chars().collect::<Vec<_>>();

    let mut idx = 0;

    while idx < chars.len() {
        let mut op_len = Range::new(1, chars.len() - idx + 1);
        match op_type.sample(rng) {
            // Retain
            0 => {
                let len = op_len.sample(rng);
                let mut bytes = 0;
                for c in chars[idx..(idx + len)].iter() {
                    bytes += c.len_utf8();
                }
                ret.retain(bytes);
                idx += len;
            }
            // Insert
            1 => {
                let len = rng.gen_range(0, 10);
                ret.insert(random_string(rng, len));
            }
            // Delete
            2 => {
                let len = op_len.sample(rng);
                let mut bytes = 0;
                for c in chars[idx..(idx + len)].iter() {
                    bytes += c.len_utf8();
                }
                ret.delete(bytes);
                idx += len;
            }
            _ => unreachable!(),
        }
    }

    ret
}
