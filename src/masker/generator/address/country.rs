use fake::locales::EN;
use fake::Fake;

use crate::masker::generator::{GeneratedValue, Generator, GeneratorError, Options};

pub struct CountryNameGenerator {}

impl CountryNameGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Generator for CountryNameGenerator {
    fn generate(&self, _: &Options) -> Result<GeneratedValue, GeneratorError> {
        use fake::faker::address::raw::*;
        Ok(GeneratedValue::String(CountryName(EN).fake::<String>()))
    }
}

pub struct CountryCodeGenerator {}

impl CountryCodeGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Generator for CountryCodeGenerator {
    fn generate(&self, _: &Options) -> Result<GeneratedValue, GeneratorError> {
        use fake::faker::address::raw::*;
        Ok(GeneratedValue::String(CountryCode(EN).fake::<String>()))
    }
}
