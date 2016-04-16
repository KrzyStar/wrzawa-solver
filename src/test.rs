extern crate rand;

use std::collections::HashMap;
use rand::Rng;

use structure::*;

pub fn generate_input<'a>() -> InputData<'a> {
    let mut activity_map = HashMap::new();
    let activities = vec![("Szydełkowanie", Module::Wyczyn, Block::B1),
                          ("Kolaborowanie z Chorągwią", Module::Wyczyn, Block::B3),
                          ("Obejmowanie wzrokiem", Module::Wyczyn, Block::B5),

                          ("Prowadzenie z mapą", Module::Spoleczny, Block::B2),
                          ("Wywieranie wpływu na ludzi", Module::Spoleczny, Block::B4),
                          ("Wygadywanie mądrych rzeczy", Module::Spoleczny, Block::B4),

                          ("Strzelanie z procy", Module::Sluzba, Block::B1),
                          ("Otwieranie puszek", Module::Sluzba, Block::B3),

                          ("Puszczanie latawców", Module::Zawod, Block::B1),
                          ("Ocenianie sposobów oceniania", Module::Zawod, Block::B2),
                          ("Kolorowanie kolorowanek", Module::Zawod, Block::B3),
                          ("Narzekanie bez sensu", Module::Zawod, Block::B5)];

    for (id, activity) in activities.iter().enumerate() {
        activity_map.insert(id as ActivityId,
                            Activity {
                                id: id as ActivityId,
                                name: activity.0,
                                module: activity.1,
                                block: activity.2,
                                min_person_count: rand::random::<u8>() % 5 + 1,
                            });
    }

    let mut person_map = HashMap::new();
    let names = vec!["Plato Stephens",
                     "Armand Mercer",
                     "Kaye Glover",
                     "Hall Humphrey",
                     "Charissa Schwartz",
                     "Oliver Crane",
                     "Brynne Barrett",
                     "Fuller Shelton",
                     "Bryar Perry",
                     "Britanni Howell",
                     "Jelani Bryan",
                     "Giselle Haley",
                     "Grady Palmer",
                     "Jermaine Klein",
                     "Aline Holder",
                     "Ralph Pruitt",
                     "Wade Mayo",
                     "Chester Cooke",
                     "Benedict Kent",
                     "Dante Kennedy"];

    for (id, name) in names.iter().enumerate() {
        let mh_count = rand::random::<u8>() % 5;
        let mh_modules = &mut Module::values();
        let mut rng = rand::thread_rng();
        rng.shuffle(mh_modules);

        let mut preferences = vec![];

        for mh_module in Module::values().into_iter().take(mh_count as usize) {
            let mh_module_activities = activity_map.values()
                                                   .filter(|x| x.module == mh_module)
                                                   .collect::<Vec<&Activity>>();
            let mh_activity = rng.choose(mh_module_activities.as_slice()).unwrap();
            preferences.push(PersonPreference::new(mh_activity.id, Preference::MustHave));
        }

        person_map.insert(id as PersonId,
                          Person::new(id as PersonId, name, preferences));
    }

    InputData::new(activity_map, person_map)
}
