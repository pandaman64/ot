use ot::selection::linewise::*;
use super::rand;

pub use super::linewise::random_lines;

use rand::distributions::{Range, Sample};
pub fn random_selection<R: rand::Rng>(rng: &mut R, num_selection: usize, base: &Vec<String>) -> Vec<Selection> {
    if base.len() == 0 {
        vec![
            Selection::Cursor(Position {
                row: 0,
                col: 0,
            }),
        ]
    } else {
        let mut range = Range::new(0, base.len());
        (0..rng.gen_range(0, num_selection)).map(|_| {
            let start_row = range.sample(rng);
            let start_col = rng.gen_range(0, base[start_row].len() + 1);
            let end_row = range.sample(rng);
            let end_col = rng.gen_range(0, base[end_row].len() + 1);

            let start = Position {
                row: start_row,
                col: start_col,
            };
            let end = Position {
                row: end_row,
                col: end_col,
            };

            if start == end {
                Selection::Cursor(start)
            } else if start < end{
                Selection::Range(start, end)
            } else {
                Selection::Range(end, start)
            }
        }).collect()
    }
}

pub fn random_target<R: rand::Rng>(rng: &mut R, num_selection: usize, max_line_len: usize, len: usize) -> Target {
    let base = random_lines(rng, max_line_len, len);
    let selection = random_selection(rng, num_selection, &base);

    Target {
        base,
        selection,
    }
}

pub fn random_operation<R: rand::Rng>(rng: &mut R, num_selection: usize, target: &Target) -> Operation {
    use self::Operation::*;
    use ot::Operation;

    match rng.gen_range(0, 4) {
        0 => Nop,
        _ => {
            let op = super::linewise::random_operation(rng, &target.base);
            let applied_base = op.apply(&target.base);
            let selection = random_selection(rng, num_selection, &applied_base);
            Op(selection, op)
        },
    }
}
