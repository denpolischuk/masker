use fake::locales::EN;
use fake::Fake;

use crate::masker::generator::{GeneratedValue, Generator, GeneratorError, Options};

pub struct CityNameGenerator {}

impl CityNameGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CityNameGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for CityNameGenerator {
    fn generate(&self, _: &Options) -> Result<GeneratedValue, GeneratorError> {
        use fake::faker::address::raw::*;
        Ok(GeneratedValue::String(CityName(EN).fake::<String>()))
    }
}
