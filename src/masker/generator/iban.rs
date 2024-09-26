use iban::Iban;
use rand::seq::SliceRandom;

use crate::masker::error::{ConfigParseError, ConfigParseErrorKind};

use super::{error::GeneratorErrorKind, GeneratedValue, Generator, GeneratorError};

#[derive(Debug)]
pub struct IbanGenerator {
    country_codes: Vec<String>,
}

impl IbanGenerator {
    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, ConfigParseError> {
        let field = "country_codes";
        let codes = match yaml[field].as_sequence() {
            Some(seq) => seq
                .iter()
                .map(|arr_item| match arr_item.as_str() {
                    Some(code) => Ok(code.to_string()),
                    None => Err(ConfigParseError {
                        field: field.to_string(),
                        kind: ConfigParseErrorKind::UnexpectedFieldType,
                    }),
                })
                .collect::<Result<Vec<String>, ConfigParseError>>()?,
            None => {
                return Err(ConfigParseError {
                    field: field.to_string(),
                    kind: ConfigParseErrorKind::MissingField,
                })
            }
        };
        Ok(Self {
            country_codes: codes,
        })
    }
}

impl Generator for IbanGenerator {
    fn generate(&self, _: &super::Options) -> Result<super::GeneratedValue, super::GeneratorError> {
        let code = self.country_codes.choose(&mut rand::thread_rng()).ok_or(
            GeneratorError::new::<Self>(GeneratorErrorKind::GenerateIban),
        )?;
        Ok(GeneratedValue::String(
            Iban::rand(code, &mut rand::thread_rng())
                .map_err(|_| {
                    GeneratorError::new::<Self>(GeneratorErrorKind::GenerateIbanForCountryCode(
                        code.clone(),
                    ))
                })?
                .to_string(),
        ))
    }
}

#[test]
fn test_iban_gen() {}
