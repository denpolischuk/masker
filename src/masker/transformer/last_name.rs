use rand::Rng;

use crate::masker::transformer::{Options, Transformer};
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

pub struct LastNameTransformer {}

impl LastNameTransformer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for LastNameTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformer for LastNameTransformer {
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
