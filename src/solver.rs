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

    assign_mhs(&mut solution, &input, 1);
    debug!("{:?}\n", solution.person_assignments);

    // STAGE 2
    // Assign 2 best fitting must-haves to all persons (in random order) not covered by stage 1.
    //
    // Activities with higher numer of missing participants have precedence.
    // Same fallback rules as stage 1.
    //

    debug!("STAGE 2:");

    assign_mhs(&mut solution, &input, 2);
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

fn assign_mhs(solution: &mut Solution, input: &InputData, stage: u8) {
    let mut rng = rand::thread_rng();
    let mh_cnt_filter = |mh_cnt| (mh_cnt <= 2 && stage == 1) || (mh_cnt > 2 && stage == 2);
    let mh_persons = &mut input.persons()
                               .map(|p| (p.must_haves(), p))
                               .filter(|&(ref mhs, _)| mh_cnt_filter(mhs.len()))
                               .map(|(mhs, p)| {
                                   (mhs.iter()
                                       .map(|mh| input.get_activity(mh.activity_id))
                                       .collect::<Vec<_>>(),
                                    p)
                               })
                               .collect::<Vec<_>>();
    if stage == 2 {
        rng.shuffle(mh_persons);
    }

    for &(ref mhs, person) in mh_persons.iter() {
        if mhs.len() >= 2 {
            let mh_pair = iproduct!(mhs.iter(), mhs.iter())
                              .filter(|&(x, y)| x.id < y.id)
                              .filter(|&(x, y)| !Activity::check_collision(&vec![x], y))
                              .max_by_key(|&(x, y)| {
                                  solution.get_missing_persons_count(x) +
                                  solution.get_missing_persons_count(y)
                              });

            if let Some((mh1, mh2)) = mh_pair {
                solution.add_assignment(person.id, mh1.id);
                solution.add_assignment(person.id, mh2.id);
                continue;
            }
        }
        if let Some(mh) = mhs.iter().filter(|mh| mh.module == Module::Sluzba).next() {
            solution.add_assignment(person.id, mh.id);
        } else if let Some(mh) = rng.choose(mhs.as_slice()) {
            solution.add_assignment(person.id, mh.id);
        }
    }
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
        0
    }
}
