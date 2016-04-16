#[macro_use]
extern crate itertools;
extern crate rand;
extern crate stats;

mod structure;
mod solver;
mod test;

fn main() {
    let input = test::generate_input();
    let mut max_solution = {
        let solution = solver::solve(&input);
        let score = solver::evaluate(&input, &solution);
        (solution, score)
    };

    for _ in 0..100 {
        let solution = solver::solve(&input);
        let score = solver::evaluate(&input, &solution);
        if score > max_solution.1 {
            max_solution = (solution, score);
        }

        println!("{}", score);
    }

    println!("{:?}", max_solution);
}
