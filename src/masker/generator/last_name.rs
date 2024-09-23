use rand::Rng;

use crate::masker::generator::{Generator, Options};

use super::{generator::GeneratedValue, GeneratorError};
const RANDOM_NAMES: &[&str] = &[
    "Adams",
    "Alvarez",
    "Anderson",
    "Bailey",
    "Baker",
    "Bennett",
    "Brooks",
    "Brown",
    "Campbell",
    "Carter",
    "Castillo",
    "Chavez",
    "Clark",
    "Collins",
    "Cook",
    "Cooper",
    "Cox",
    "Cruz",
    "Davis",
    "Diaz",
    "Edwards",
    "Evans",
    "Flores",
    "Foster",
    "Garcia",
    "Gomez",
    "Gonzalez",
    "Gray",
    "Green",
    "Gutierrez",
    "Hall",
    "Harris",
    "Hernandez",
    "Hill",
    "Howard",
    "Hughes",
    "Jackson",
    "James",
    "Jimenez",
    "Johnson",
    "Jones",
    "Kelly",
    "Kim",
    "King",
    "Lee",
    "Lewis",
    "Long",
    "Lopez",
    "Martin",
    "Martinez",
    "Mendoza",
    "Miller",
    "Mitchell",
    "Moore",
    "Morales",
    "Morgan",
    "Morris",
    "Murphy",
    "Myers",
    "Nelson",
    "Nguyen",
    "Ortiz",
    "Parker",
    "Patel",
    "Perez",
    "Peterson",
    "Phillips",
    "Powell",
    "Price",
    "Ramirez",
    "Ramos",
    "Reed",
    "Reyes",
    "Richardson",
    "Rivera",
    "Roberts",
    "Robinson",
    "Rodriguez",
    "Rogers",
    "Ross",
    "Ruiz",
    "Sanchez",
    "Sanders",
    "Scott",
    "Smith",
    "Stewart",
    "Taylor",
    "Thomas",
    "Thompson",
    "Torres",
    "Turner",
    "Walker",
    "Ward",
    "Watson",
    "White",
    "Williams",
    "Wilson",
    "Wood",
    "Wright",
    "Young",
];
// To not calculate the size of array each time
const RANDOM_NAMES_SIZE: usize = RANDOM_NAMES.len();

pub struct LastNameGenerator {}

impl LastNameGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for LastNameGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for LastNameGenerator {
    fn generate(&self, _: &Options) -> Result<GeneratedValue, GeneratorError> {
        let rand_i = rand::thread_rng().gen_range(0..RANDOM_NAMES_SIZE - 1);
        let res = RANDOM_NAMES[rand_i];

        Ok(GeneratedValue::String(res.into()))
    }
}
