use std::slice::Iter;
use std::collections::hash_map::{HashMap, Values};

pub type ActivityId = u16;
pub type PersonId = u16;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Module {
    Sluzba,
    Wyczyn,
    Zawod,
    Spoleczny,
}

impl Module {
    pub fn values() -> Vec<Self> {
        vec![Module::Sluzba, Module::Wyczyn, Module::Zawod, Module::Spoleczny]
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Block {
    B1,
    B2,
    B3,
    B4,
    B5,
}

impl Block {
    pub fn snap(block: Self) -> Self {
        if block == Block::B1 || block == Block::B2 {
            Block::B1
        } else if block == Block::B3 || block == Block::B4 {
            Block::B3
        } else {
            Block::B5
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Preference {
    MustHave,
    Preferred,
    CantHave,
}

#[derive(Debug)]
pub struct Activity<'a> {
    pub id: ActivityId,
    pub name: &'a str,
    pub module: Module,
    pub block: Block,
    pub min_person_count: u8,
}

impl<'a> Activity<'a> {
    // assumes that existing assignments in v do not collide with each other
    pub fn check_collision(v: &Vec<&Self>, act: &Self) -> bool {
        v.iter().any(|x| x.module == act.module || x.block == act.block) ||
        if v.iter().any(|x| x.module == Module::Sluzba) {
            false
        } else if act.module == Module::Sluzba {
            v.iter().any(|x| Block::snap(x.block) == act.block)
        } else {
            (Block::snap(act.block) == Block::B1 &&
             v.iter().any(|x| Block::snap(x.block) == Block::B3)) ||
            (Block::snap(act.block) == Block::B3 &&
             v.iter().any(|x| Block::snap(x.block) == Block::B1))
        }
    }
}

#[derive(Debug)]
pub struct PersonPreference {
    pub activity_id: ActivityId,
    pub preference: Preference,
}

impl PersonPreference {
    pub fn new(activity_id: ActivityId, preference: Preference) -> Self {
        PersonPreference {
            activity_id: activity_id,
            preference: preference,
        }
    }
}

#[derive(Debug)]
pub struct Person<'a> {
    pub id: PersonId,
    pub name: &'a str,
    pub preferences: Vec<PersonPreference>,
}

impl<'a> Person<'a> {
    pub fn new(id: PersonId, name: &'a str, preferences: Vec<PersonPreference>) -> Self {
        Person {
            id: id,
            name: name,
            preferences: preferences,
        }
    }

    pub fn preferences(&self) -> Iter<PersonPreference> {
        self.preferences.iter()
    }
}

#[derive(Debug)]
pub struct InputData<'a> {
    activities: HashMap<ActivityId, Activity<'a>>,
    persons: HashMap<PersonId, Person<'a>>,
}

impl<'a> InputData<'a> {
    pub fn new(activity_map: HashMap<ActivityId, Activity<'a>>,
               person_map: HashMap<PersonId, Person<'a>>)
               -> Self {
        InputData {
            activities: activity_map,
            persons: person_map,
        }
    }

    pub fn activities(&self) -> Values<ActivityId, Activity> {
        self.activities.values()
    }

    pub fn persons(&self) -> Values<PersonId, Person> {
        self.persons.values()
    }

    pub fn get_activity(&self, activity_id: ActivityId) -> &Activity {
        self.activities.get(&activity_id).unwrap()
    }
}

#[derive(Debug)]
pub struct Solution {
    pub person_assignments: HashMap<PersonId, Vec<ActivityId>>,
    pub activity_assignments: HashMap<ActivityId, Vec<PersonId>>,
}

impl Solution {
    pub fn new(p_assigns: HashMap<PersonId, Vec<ActivityId>>,
               a_assigns: HashMap<ActivityId, Vec<PersonId>>)
               -> Self {
        Solution {
            person_assignments: p_assigns,
            activity_assignments: a_assigns,
        }
    }

    pub fn get_person_assignments(&mut self, person_id: PersonId) -> &mut Vec<ActivityId> {
        self.person_assignments.entry(person_id).or_insert(vec![])
    }

    pub fn get_activity_assignments(&mut self, activity_id: ActivityId) -> &mut Vec<PersonId> {
        self.activity_assignments.entry(activity_id).or_insert(vec![])
    }

    pub fn add_assignment(&mut self, person_id: PersonId, activity_id: ActivityId) {
        self.get_person_assignments(person_id).push(activity_id);
        self.get_activity_assignments(activity_id).push(person_id);
    }

    pub fn get_missing_persons_count(&mut self, activity: &Activity) -> i8 {
        let a_assigns = self.get_activity_assignments(activity.id);
        (activity.min_person_count as i8) - (a_assigns.len() as i8)
    }
}
