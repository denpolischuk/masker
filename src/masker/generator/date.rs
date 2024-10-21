use chrono::{DateTime, Utc};
use fake::{
    faker::{
        self,
        chrono::{
            ar_sa::DateTime,
            en::{DateTime, DateTimeBefore, DateTimeBetween},
        },
    },
    locales::EN,
    Fake, Faker,
};

use crate::masker::error::{ConfigParseError, ConfigParseErrorKind};

use super::{GeneratedValue, Generator, GeneratorError, Options};

pub struct DateTimeGenerator {
    before: Option<DateTime<chrono::Utc>>,
    after: Option<DateTime<chrono::Utc>>,
}

impl DateTimeGenerator {
    fn parse_date_from_str(
        datestr: &str,
        field: &str,
    ) -> Result<DateTime<chrono::Utc>, ConfigParseError> {
        Ok(chrono::DateTime::parse_from_rfc3339(datestr)
            .map_err(|e| ConfigParseError {
                field: field.to_string(),
                kind: ConfigParseErrorKind::FailedToCreateGeneratorFromConfig(
                    GeneratorError::new::<Self>(
                        super::error::GeneratorErrorKind::WrongDateTimeFormatForDateGenerator(e),
                    ),
                ),
            })?
            .to_utc())
    }

    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, ConfigParseError> {
        let field = "before";
        let before = match yaml[field].as_str() {
            Some(datestr) => Some(Self::parse_date_from_str(datestr, field)?),
            None => None,
        };
        let after = match yaml[field].as_str() {
            Some(datestr) => Some(Self::parse_date_from_str(datestr, field)?),
            None => None,
        };
        Ok(Self { before, after })
    }
}

impl Generator for DateTimeGenerator {
    fn generate(&self, _: &Options) -> std::result::Result<super::GeneratedValue, GeneratorError> {
        use fake::faker::chrono::raw::*;
        if let Some(b_date) = self.before {
            if let Some(a_date) = self.after {
                return Ok(GeneratedValue::String(
                    DateTimeBetween(EN, b_date, a_date)
                        .fake::<chrono::DateTime<Utc>>()
                        .to_string(),
                ));
            }
            return Ok(GeneratedValue::String(
                DateTimeBefore(EN, b_date)
                    .fake::<chrono::DateTime<Utc>>()
                    .to_string(),
            ));
        }
        if let Some(a_date) = self.after {
            return Ok(GeneratedValue::String(
                DateTimeAfter(EN, a_date)
                    .fake::<chrono::DateTime<Utc>>()
                    .to_string(),
            ));
        }
        Ok(GeneratedValue::String(
            DateTime(EN).fake::<chrono::DateTime<Utc>>().to_string(),
        ))
    }
}
