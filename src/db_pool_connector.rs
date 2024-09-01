use mysql::*;

pub fn get_pool(conn_str: String) -> Result<Pool, mysql::Error> {
    let opts = Opts::from_url(conn_str.as_str())?;
    Pool::new(opts)
}
