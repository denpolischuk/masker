use fake::locales::EN;
use fake::Fake;

use crate::masker::generator::{GeneratedValue, Generator, GeneratorError, Options};

pub struct StateNameGenerator {}

impl StateNameGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Generator for StateNameGenerator {
    fn generate(&self, _: &Options) -> Result<GeneratedValue, GeneratorError> {
        use fake::faker::address::raw::*;
        Ok(GeneratedValue::String(StateName(EN).fake::<String>()))
    }
}
