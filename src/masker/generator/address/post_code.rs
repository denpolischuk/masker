use fake::locales::EN;
use fake::Fake;

use crate::masker::generator::{GeneratedValue, Generator, GeneratorError, Options};

pub struct PostCodeGenerator {}

impl PostCodeGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Generator for PostCodeGenerator {
    fn generate(&self, _: &Options) -> Result<GeneratedValue, GeneratorError> {
        use fake::faker::address::raw::*;
        Ok(GeneratedValue::String(PostCode(EN).fake::<String>()))
    }
}
