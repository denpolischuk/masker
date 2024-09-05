mod database;
mod masker;

#[tokio::main]
async fn main() {
    let f = std::fs::File::open("schema.yaml").unwrap();
    let yaml = serde_yaml::from_reader(f).unwrap();
    let masker = std::sync::Arc::new(masker::Masker::new_from_yaml(&yaml).unwrap());
    let db = database::new_db_adapter_from_yaml(&yaml).unwrap();

    db.apply_mask(masker.clone()).await.unwrap();
}
