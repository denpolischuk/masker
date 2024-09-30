use fake::locales::EN;
use fake::Fake;

use crate::masker::generator::{GeneratedValue, Generator, GeneratorError, Options};

pub struct StreetNameGenerator {}

impl StreetNameGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Generator for StreetNameGenerator {
    fn generate(&self, _: &Options) -> Result<GeneratedValue, GeneratorError> {
        use fake::faker::address::raw::*;
        Ok(GeneratedValue::String(StreetName(EN).fake::<String>()))
    }
}
