use rand::Rng;

use crate::masker::generator::{Generator, Options};

use super::{generator::GeneratedValue, GeneratorError};
const RANDOM_NAMES: &[&str] = &[
    "Alice", "Bob", "Charlie", "Diana", "Ethan", "Fiona", "George", "Hannah", "Isaac", "Jasmine",
    "Kevin", "Lena", "Mason", "Nina", "Oscar", "Piper", "Quinn", "Ruby", "Sam", "Tina", "Ulysses",
    "Vera", "Wade", "Xena", "Yara", "Zane", "Aaron", "Bella", "Caleb", "Delia", "Elijah", "Faith",
    "Gavin", "Hazel", "Ivan", "Julia", "Kyle", "Lila", "Max", "Nora", "Owen", "Paige", "Quincy",
    "Riley", "Sophie", "Trent", "Uma", "Vince", "Willow", "Xander", "Yasmin", "Zoe", "Adam",
    "Brooke", "Cody", "Daisy", "Eli", "Freya", "Grayson", "Holly", "Isaiah", "Jade", "Kara",
    "Liam", "Miles", "Naomi", "Orion", "Penny", "Quentin", "Rachel", "Sean", "Tara", "Ulrich",
    "Violet", "Wesley", "Xavier", "Yvonne", "Zara", "Asher", "Brianna", "Colin", "Derek", "Emily",
    "Finn", "Giselle", "Hunter", "Isla", "Jake", "Kendall", "Logan", "Maya", "Noah", "Olivia",
    "Preston",
];
// To not calculate the size of array each time
const RANDOM_NAMES_SIZE: usize = RANDOM_NAMES.len();

pub struct FirstNameGenerator {}

impl FirstNameGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for FirstNameGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for FirstNameGenerator {
    fn generate(&self, _: &Options) -> Result<GeneratedValue, GeneratorError> {
        let rand_i = rand::thread_rng().gen_range(0..RANDOM_NAMES_SIZE - 1);
        let res = RANDOM_NAMES[rand_i];

        Ok(GeneratedValue::String(res.into()))
    }
}
