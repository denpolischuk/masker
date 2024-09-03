mod database;
mod masker;

fn main() {
    let f = std::fs::File::open("schema.yaml").unwrap();
    let yaml = serde_yaml::from_reader(f).unwrap();
    let masker = masker::Masker::new_from_yaml(&yaml).unwrap();
    let db = database::new_db_adapter_from_yaml(&yaml).unwrap();

    db.apply_mask(&masker).unwrap();
}
