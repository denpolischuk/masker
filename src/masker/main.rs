use crate::masker::Entity;

pub struct Masker {
    entities: Vec<Entity>,
}

impl Masker {
    pub fn new(entities: Vec<Entity>) -> Self {
        Masker { entities }
    }

    pub fn new_from_yaml(yaml: &serde_yaml::Value) -> Result<Self, Box<dyn std::error::Error>> {
        let mut schemas: Vec<Entity> = vec![];
        match yaml["schemas"].as_sequence() {
            Some(seq) => {
                seq.iter()
                    .try_for_each(|schema_yaml| -> Result<(), Box<dyn std::error::Error>> {
                        schemas.push(Entity::new_from_yaml(schema_yaml)?);
                        Ok(())
                    })
            }
            None => return Err("Missing schemas definition in yaml.".into()),
        }?;
        Ok(Masker::new(schemas))
    }

    pub fn get_entities(&self) -> &Vec<Entity> {
        &self.entities
    }
}
