use crate::masker::transformer::{FirstNameTransformer, LastNameTransformer, TemplateTransformer};

pub trait Transformer {
    fn read_parameters_from_yaml(
        &mut self,
        yaml: &serde_yaml::Value,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn generate(&self, options: Options) -> Result<mysql::Value, Box<dyn std::error::Error>>;
}

pub struct Options {
    pub pk: Box<dyn ToString>,
}

pub fn new_from_yaml(
    yaml: &serde_yaml::Value,
) -> Result<Box<dyn Transformer>, Box<dyn std::error::Error>> {
    match yaml["kind"].as_str() {
        Some(s) => match s {
            "FirstName" => Ok(Box::new(FirstNameTransformer::new())),
            "LastName" => Ok(Box::new(LastNameTransformer::new())),
            "Template" => {
                let mut tr = Box::new(TemplateTransformer::default());
                tr.read_parameters_from_yaml(yaml)?;
                Ok(tr)
            }
            "MobilePhone" => todo!(),
            _ => Err(format!("Found unknown field kind {}", s).into()),
        },
        None => Err(
            "Tried to read entry from a fields list, but couldn't locate 'kind' property".into(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::masker::transformer::new_from_yaml;

    #[test]
    fn get_transformer_from_yaml() -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::from_str("kind: LastName").unwrap();
        _ = new_from_yaml(&yaml)?;
        Ok(())
    }

    #[test]
    fn fail_on_missing_kind() {
        let yaml = serde_yaml::from_str("kind: SomethingElse").unwrap();
        match new_from_yaml(&yaml) {
            Ok(_) => panic!("Expected error, but got Ok(Transformer) instead"),
            Err(e) => assert!(e.to_string().contains("unknown field kind")),
        }
    }
    #[test]
    fn fail_on_unknown_kind() {
        let yaml = serde_yaml::from_str("some_other_key: SomethingElse").unwrap();
        match new_from_yaml(&yaml) {
            Ok(_) => panic!("Expected error, but got Ok(Transformer) instead"),
            Err(e) => assert!(e.to_string().contains("couldn't locate 'kind'")),
        }
    }
}
