use super::{GeneratedValue, Generator, GeneratorError, Options};
type GeneratorFunction = fn(&Options) -> Result<GeneratedValue, GeneratorError>;
pub struct SimpleGenerator {
    generator: GeneratorFunction,
}

// This is a wrapper struct that implements Generator trait for simple generator functions, like
// first name, last name and so on, that don't require any configuration from the user.
// These functions are simple thus are just implemented as public functions and can be re-used. The
// wrapper is used for the type consistency in a global masker object.
impl SimpleGenerator {
    pub fn new(generator_fn: GeneratorFunction) -> Self {
        Self {
            generator: generator_fn,
        }
    }
}

impl Generator for SimpleGenerator {
    fn generate(&self, options: &Options) -> Result<GeneratedValue, super::GeneratorError> {
        (self.generator)(options)
    }
}
