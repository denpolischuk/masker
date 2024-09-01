use rand::Rng;

use crate::masker::transformer::{Options, Transformer};
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

pub struct FirstNameTransformer {}

impl FirstNameTransformer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for FirstNameTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformer for FirstNameTransformer {
    fn generate(&self, _: Options) -> Result<mysql::Value, Box<dyn std::error::Error>> {
        let rand_i = rand::thread_rng().gen_range(0..RANDOM_NAMES_SIZE - 1);
        let res = RANDOM_NAMES[rand_i];

        // // If it happens that random name is the same as randomly picked one, then use the next
        // // name in the list
        // if value.to_lowercase().trim() == res.to_lowercase().trim() {
        //     return Ok(String::from(RANDOM_NAMES[rand_i + 1]));
        // }
        Ok(mysql::Value::Bytes(res.into()))
    }

    fn read_parameters_from_yaml(
        &mut self,
        _: &serde_yaml::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
