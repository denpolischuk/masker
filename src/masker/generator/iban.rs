use std::collections::HashMap;

use iban::Iban;
use rand::seq::SliceRandom;

use crate::masker::error::{ConfigParseError, ConfigParseErrorKind};

use super::{error::GeneratorErrorKind, GeneratedValue, Generator, GeneratorError, Options};

#[derive(Debug)]
pub struct IbanGenerator {
    formatted: bool,
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
        let formatted = yaml["formatted"].as_bool().unwrap_or(false);
        let allowed_codes = iban::get_supported_countries();
        if let Some(not_found) = codes
            .iter()
            .rfind(|code| !allowed_codes.contains(&code.as_str()))
        {
            return Err(ConfigParseError {
                field: field.to_string(),
                kind: ConfigParseErrorKind::FailedToCreateGeneratorFromConfig(
                    GeneratorError::new::<Self>(GeneratorErrorKind::UnexpectedCountryCodeForIban(
                        not_found.clone(),
                    )),
                ),
            });
        }
        Ok(Self {
            country_codes: codes,
            formatted,
        })
    }
}

impl Generator for IbanGenerator {
    fn generate(&self, _: &super::Options) -> Result<super::GeneratedValue, super::GeneratorError> {
        let code = self.country_codes.choose(&mut rand::thread_rng()).ok_or(
            GeneratorError::new::<Self>(GeneratorErrorKind::GenerateIban),
        )?;
        let iban = Iban::rand(code, &mut rand::thread_rng()).map_err(|_| {
            GeneratorError::new::<Self>(GeneratorErrorKind::GenerateIbanForCountryCode(
                code.clone(),
            ))
        })?;
        Ok(GeneratedValue::String(if self.formatted {
            iban.to_string()
        } else {
            iban.as_str().to_string()
        }))
    }
}

#[test]
fn parse_generator_from_yaml() {
    let valid_yaml = "country_codes:
    - DE
    - FR";
    let yaml: serde_yaml::Value = serde_yaml::from_str(valid_yaml).unwrap();
    IbanGenerator::new_from_yaml(&yaml).unwrap();
}

#[test]
fn fails_to_create_generator_when_unexpected_country_code_is_met() {
    let valid_yaml = "country_codes:
    - DE
    - non-valid";
    let yaml: serde_yaml::Value = serde_yaml::from_str(valid_yaml).unwrap();
    let err = IbanGenerator::new_from_yaml(&yaml).unwrap_err();
    assert_eq!(
        err.kind,
        ConfigParseErrorKind::FailedToCreateGeneratorFromConfig(
            GeneratorError::new::<IbanGenerator>(GeneratorErrorKind::UnexpectedCountryCodeForIban(
                "non-valid".to_string(),
            )),
        )
    );
}

#[test]
fn generates_random_iban() {
    let valid_yaml = "country_codes:
    - DE";
    let yaml: serde_yaml::Value = serde_yaml::from_str(valid_yaml).unwrap();
    let generator = IbanGenerator::new_from_yaml(&yaml).unwrap();
    let options: Options = HashMap::new();
    if let GeneratedValue::String(iban) = generator.generate(&options).unwrap() {
        let r = regex::Regex::new(r"DE\d{20}").unwrap();
        assert!(r.is_match(iban.as_str()))
    } else {
        panic!("expected IBAN as string, got Number");
    }
}

#[test]
fn generates_random_iban_formatted() {
    let valid_yaml = "country_codes:
    - DE
formatted: true";
    let yaml: serde_yaml::Value = serde_yaml::from_str(valid_yaml).unwrap();
    let generator = IbanGenerator::new_from_yaml(&yaml).unwrap();
    let options: Options = HashMap::new();
    if let GeneratedValue::String(iban) = generator.generate(&options).unwrap() {
        let r = regex::Regex::new(r"DE\d{2} \d{4} \d{4} \d{4} \d{4} \d{2}").unwrap();
        assert!(r.is_match(iban.as_str()))
    } else {
        panic!("expected IBAN as string, got Number");
    }
}
