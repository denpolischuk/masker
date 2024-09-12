use crate::masker::{
    error::{ConfigParseError, ConfigParseErrorKind},
    transformer::{Options, Transformer},
};

use super::{tranformer::GeneratedValue, TransformerError};
pub struct TemplateTransformer {
    template: String,
}

impl TemplateTransformer {
    pub fn new(template: String) -> TemplateTransformer {
        Self {
            template: template.clone(),
        }
    }

    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, ConfigParseError> {
        match yaml["template"].as_str() {
            Some(t) => Ok(Self {
                template: t.to_string(),
            }),
            None => Err(ConfigParseError {
                field: "template".to_string(),
                kind: ConfigParseErrorKind::MissingField,
            }),
        }
    }
}

impl Default for TemplateTransformer {
    fn default() -> Self {
        Self::new(String::from("example.com"))
    }
}

impl Transformer for TemplateTransformer {
    fn generate(&self, opts: &Options) -> Result<GeneratedValue, TransformerError> {
        let res = self.template.replace("{id}", opts.pk.to_string().as_str());
        Ok(GeneratedValue::String(res))
    }
}
