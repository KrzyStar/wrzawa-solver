extern crate rand;
extern crate stats;

use std::collections::HashMap;
use rand::Rng;

use structure::*;

macro_rules! debug {
	($fmt:expr) => {{ if debug_flag() { println!($fmt) } }};
	($fmt:expr, $($arg:tt)*) => {{ if debug_flag() { println!($fmt, $($arg)*) } }};
}

fn debug_flag() -> bool {
    false
}

pub fn solve(input: &InputData) -> Solution {
    let mut solution = Solution::new(HashMap::new(), HashMap::new());
    let mut rng = rand::thread_rng();

    // STAGE 1
    // Assign chosen must-haves to persons with 2 or less such preferences.
    //
    // In case of collision will fall back to random selected MH (Sluzba activities have
    // precedence).
    // Ideally, such case should be rejected by the user interface.
    //

    debug!("STAGE 1:");

    for person in input.persons() {
        let must_haves = person.preferences()
                               .filter(|x| x.preference == Preference::MustHave)
                               .map(|x| input.get_activity(x.activity_id))
                               .collect::<Vec<&Activity>>();

        if must_haves.len() == 2 {
            if Activity::check_collision(&vec![must_haves[0]], must_haves[1]) {
                if must_haves[0].module == Module::Sluzba {
                    solution.add_assignment(person.id, must_haves[0].id);
                } else if must_haves[1].module == Module::Sluzba {
                    solution.add_assignment(person.id, must_haves[1].id);
                } else {
                    let mh_choice = rng.choose(must_haves.as_slice()).unwrap();
                    solution.add_assignment(person.id, mh_choice.id)
                }
            } else {
                solution.add_assignment(person.id, must_haves[0].id);
                solution.add_assignment(person.id, must_haves[1].id);
            }
        } else if must_haves.len() == 1 {
            solution.add_assignment(person.id, must_haves[0].id);
        }
    }

    debug!("{:?}\n", solution.person_assignments);

    // STAGE 2
    // Assign 2 best fitting must-haves to all persons (in random order) not covered by stage 1.
    //
    // Activities with higher numer of missing participants have precedence.
    // Same fallback rules as stage 1.
    //

    debug!("STAGE 2:");

    let persons = &mut input.persons().collect::<Vec<&Person>>();
    rng.shuffle(persons);

    for person in persons {
        let must_haves = person.preferences()
                               .filter(|x| x.preference == Preference::MustHave)
                               .map(|x| input.get_activity(x.activity_id))
                               .collect::<Vec<&Activity>>();

        if must_haves.len() > 2 {
            let mh_pair = iproduct!(must_haves.iter(), must_haves.iter())
                              .filter(|&(x, y)| x.id < y.id)
                              .filter(|&(x, y)| !Activity::check_collision(&vec![x], y))
                              .max_by_key(|&(x, y)| {
                                  solution.get_missing_persons_count(x) +
                                  solution.get_missing_persons_count(y)
                              });

            match mh_pair {
                Some((mh1, mh2)) => {
                    solution.add_assignment(person.id, mh1.id);
                    solution.add_assignment(person.id, mh2.id);
                }

                None => {
                    match must_haves.iter().filter(|x| x.module == Module::Sluzba).next() {
                        Some(mh) => solution.add_assignment(person.id, mh.id),
                        None => {
                            let mh_choice = rng.choose(must_haves.as_slice()).unwrap();
                            solution.add_assignment(person.id, mh_choice.id)
                        }
                    }
                }
            }
        }
    }

    debug!("{:?}\n", solution.person_assignments);

    // STAGE 3
    // Fill in missing assignments, so that every person has exactly 2 activities.
    //
    // This is done in 2 passes, initially making sure that everybody has at least one.
    // MH/Preferred activities are considered first.
    //

    // STAGE 3 - PASS 1
    //

    debug!("STAGE 3 - PASS 1");

    let persons = &mut input.persons()
                            .filter(|x| solution.get_person_assignments(x.id).len() == 0)
                            .collect::<Vec<&Person>>();
    rng.shuffle(persons);

    for person in persons {
        assign_activity(&mut solution, &input, person)
    }

    debug!("{:?}\n", solution.person_assignments);

    // STAGE 3 - PASS 2
    //

    debug!("STAGE 3 - PASS 2");

    let persons = &mut input.persons()
                            .filter(|x| solution.get_person_assignments(x.id).len() == 1)
                            .collect::<Vec<&Person>>();
    rng.shuffle(persons);

    for person in persons {
        assign_activity(&mut solution, &input, person);
    }

    debug!("{:?}\n", solution.person_assignments);

    // STAGE 4
    // Assign remaining activities. Works pretty much the same as stage 3.
    //

    // STAGE 4 - PASS 1
    //

    debug!("STAGE 4 - PASS 1");

    let persons = &mut input.persons().collect::<Vec<&Person>>();
    rng.shuffle(persons);

    for person in persons {
        assign_activity(&mut solution, &input, person);
    }

    debug!("{:?}\n", solution.person_assignments);

    // STAGE 4 - PASS 2
    //

    debug!("STAGE 4 - PASS 2");

    let persons = &mut input.persons().collect::<Vec<&Person>>();
    rng.shuffle(persons);

    for person in persons {
        assign_activity(&mut solution, &input, person);
    }

    debug!("{:?}\n", solution.person_assignments);

    solution
}

fn assign_activity(solution: &mut Solution, input: &InputData, person: &Person) {
    let assigned = solution.get_person_assignments(person.id)
                           .iter()
                           .map(|&x| input.get_activity(x))
                           .collect::<Vec<&Activity>>();

    let preferred = person.preferences()
                          .filter(|x| {
                              x.preference == Preference::MustHave ||
                              x.preference == Preference::Preferred
                          })
                          .map(|x| input.get_activity(x.activity_id))
                          .filter(|x| !Activity::check_collision(&assigned, x))
                          .max_by_key(|x| solution.get_missing_persons_count(x));

    match preferred {
        Some(activity) => solution.add_assignment(person.id, activity.id),

        None => {
            let cant_have = person.preferences()
                                  .filter(|x| x.preference == Preference::CantHave)
                                  .map(|x| input.get_activity(x.activity_id))
                                  .collect::<Vec<&Activity>>();

            let activity = input.activities()
                                .filter(|x| !Activity::check_collision(&assigned, x))
                                .filter(|x| cant_have.iter().all(|y| x.id != y.id))
                                .max_by_key(|x| solution.get_missing_persons_count(x))
                                .unwrap();

            solution.add_assignment(person.id, activity.id);
        }
    }
}

pub fn evaluate(input: &InputData, solution: &Solution) -> f64 {
    let person_scores = input.persons()
                             .map(|x| evaluate_person(x, solution))
                             .collect::<Vec<_>>();
    let person_mean = stats::mean(person_scores.iter().cloned());
    let person_stddev = stats::stddev(person_scores.iter().cloned());

    let activity_mean = stats::mean(input.activities().map(|x| evaluate_activity(x, solution)));

    debug!("Person scores: {:?}", person_scores);
    debug!("Activity mean: {}", activity_mean);

    (person_mean - person_stddev) * activity_mean * 20.0
}

// find total sum of preference scores - 2 for MH, 1 for preferred
fn evaluate_person(person: &Person, solution: &Solution) -> u8 {
    let mh_pref_it = person.preferences()
                           .filter(|x| {
                               x.preference == Preference::MustHave ||
                               x.preference == Preference::Preferred
                           })
                           .filter(|x| {
                               solution.person_assignments[&person.id].contains(&x.activity_id)
                           });

    mh_pref_it.map(|x| {
                  if x.preference == Preference::MustHave {
                      2
                  } else {
                      1
                  }
              })
              .fold(0, |acc, x| acc + x)
}

fn evaluate_activity(activity: &Activity, solution: &Solution) -> u8 {
    if solution.activity_assignments[&activity.id].len() as u8 >= activity.min_person_count {
        1
    } else {
        println!("dong");
        0
    }
}
