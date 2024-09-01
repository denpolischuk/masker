use crate::masker::transformer::{Options, Transformer};
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
    fn generate(&self, opts: Options) -> Result<mysql::Value, Box<dyn std::error::Error>> {
        let res = self.template.replace("{id}", opts.pk.to_string().as_str());
        Ok(mysql::Value::Bytes(res.into()))
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
