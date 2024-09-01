use mysql::prelude::*;
pub mod db_pool_connector;
pub mod masker;

fn main() {
    let db_masker = masker::Masker::new_from_yaml("schema.yaml").unwrap();
    let pool = match db_pool_connector::get_pool(db_masker.get_conn_str()) {
        Ok(p) => p,
        Err(e) => panic!("Couldn't create a connection pool: {}", e),
    };
    let mut c = pool.get_conn().expect("Couldn't establish a connection.");
    let dbs: Vec<String> = c.query("SHOW TABLES;").unwrap();
    let expected_tables: Vec<String> = db_masker
        .entities
        .iter()
        .map(|schema| schema.get_table_name())
        .collect();

    for schema in db_masker.entities.iter() {
        _ = schema
            .mask(
                pool.get_conn()
                    .expect("Couldn't retrieve DB connection from the pool"),
            )
            .unwrap()
    }

    for r in dbs.iter() {
        println!("Found tables - {}", r.as_str());
        println!("Expected tables - {:?}", expected_tables);
    }
}
