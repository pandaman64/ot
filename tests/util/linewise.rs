use ot::linewise::*;
use super::rand;

use super::charwise as charwise_util;

pub fn random_lines<R: rand::Rng>(
    rng: &mut R,
    max_line_len: usize,
    line_num: usize,
) -> Vec<String> {
    use rand::distributions::{Range, Sample};

    let mut len = Range::new(0, max_line_len + 1);
    (0..line_num)
        .map(|_| {
            let len = len.sample(rng);
            charwise_util::random_string(rng, len)
        })
        .collect()
}

pub fn random_operation<R: rand::Rng>(rng: &mut R, original: &[String]) -> Operation {
    use rand::distributions::{Range, Sample};

    let mut op_type = Range::new(0, 4);
    let mut ret = Operation::new();

    let mut idx = 0;

    while idx < original.len() {
        let mut op_len = Range::new(1, original.len() - idx + 1);
        match op_type.sample(rng) {
            // Retain
            0 => {
                let len = op_len.sample(rng);
                ret.retain(len);
                idx += len;
            }
            // Insert
            1 => {
                let len = rng.gen_range(0, 10);
                ret.insert(charwise_util::random_string(rng, len));
            }
            // Modify
            2 => {
                ret.modify(charwise_util::random_operation(rng, &original[idx]));
                idx += 1;
            }
            // Delete
            3 => {
                let len = op_len.sample(rng);
                ret.delete(len);
                idx += len;
            }
            _ => unreachable!(),
        }
    }

    ret
}
