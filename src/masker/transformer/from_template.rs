use crate::masker::transformer::{Options, Transformer};

use super::tranformer::GeneratedValue;
pub struct TemplateTransformer {
    template: String,
}

impl TemplateTransformer {
    pub fn new(template: String) -> TemplateTransformer {
        Self {
            template: template.clone(),
        }
    }
}

impl Default for TemplateTransformer {
    fn default() -> Self {
        Self::new(String::from("example.com"))
    }
}

impl Transformer for TemplateTransformer {
    fn generate(&self, opts: &Options) -> Result<GeneratedValue, Box<dyn std::error::Error>> {
        let res = self.template.replace("{id}", opts.pk.to_string().as_str());
        Ok(GeneratedValue::String(res))
    }

    fn read_parameters_from_yaml(
        &mut self,
        yaml: &serde_yaml::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match yaml["template"].as_str() {
            Some(t) => self.template = t.to_string(),
            None => return Err("Couldn't find a 'template' property for Template kind.".into()),
        }

        Ok(())
    }
}
