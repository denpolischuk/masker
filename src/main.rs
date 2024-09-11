use std::process::exit;

mod database;
mod masker;

#[tokio::main]
async fn main() {
    let f = std::fs::File::open("schema.yaml").expect("couldn't read config file");
    let yaml = match serde_yaml::from_reader(f) {
        Ok(y) => y,
        Err(e) => {
            println!("couldn't parse config file: {}", e);
            exit(1);
        }
    };
    let masker = std::sync::Arc::new(match masker::Masker::new_from_yaml(&yaml) {
        Ok(m) => m,
        Err(e) => {
            println!("couldn't create masker entity: {e}");
            exit(1);
        }
    });
    let db = match database::new_db_adapter_from_yaml(&yaml) {
        Ok(db) => db,
        Err(e) => {
            println!("couldn't create DB adapter instance: {e}");
            exit(1);
        }
    };

    match db.apply_mask(masker.clone()).await {
        Ok(_) => (),
        Err(e) => {
            println!("couldn't mask the schema correctly: {e}");
            exit(1);
        }
    };
}
