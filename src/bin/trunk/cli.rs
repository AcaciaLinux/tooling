use std::{io::Read, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use clap::Parser;
use http::StatusCode;
use tooling::{
    error::{Error, ErrorExt, ErrorType},
    model::{Home, ObjectDB, ObjectID},
    ODB_DEPTH,
};

use log::error;

/// The AcaciaLinux server
#[derive(Parser)]
pub struct Cli {
    /// The log level to operate on (0 = info, 1 = debug, * = trace)
    #[arg(long = "loglevel", short = 'v', default_value_t = 0, global = true)]
    pub loglevel: u8,

    /// The address to run on
    #[arg(long, default_value_t = String::from("0.0.0.0"))]
    pub address: String,

    /// The port to run on
    #[arg(long, default_value_t = 27015)]
    pub port: u16,

    /// The home directory where all Acacia tooling works in [~/.acacia]
    #[arg(long)]
    home: Option<PathBuf>,
}

impl Cli {
    pub async fn run(&self) -> Result<i32, Error> {
        if std::env::var("RUST_LOG").is_err() {
            match &self.loglevel {
                0 => {}
                1 => std::env::set_var("RUST_LOG", "info"),
                2 => std::env::set_var("RUST_LOG", "debug"),
                _ => std::env::set_var("RUST_LOG", "trace"),
            }
        }
        pretty_env_logger::init();

        let home = self.get_home()?;

        let odb = ObjectDB::init(home.object_db_path(), ODB_DEPTH)?;

        let app = Router::new()
            .route("/object/:oid", get(get_object))
            .with_state(Arc::new(odb));

        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.address, self.port))
            .await
            .ctx(|| "Binding trunk server socket")?;
        axum::serve(listener, app).await.ctx(|| "Running server")?;

        Ok(0)
    }

    pub fn get_home(&self) -> Result<Home, Error> {
        let home = match &self.home {
            Some(root) => Home::new(root.clone()),
            None => match home::home_dir() {
                Some(home_dir) => Home::new(home_dir.join(tooling::HOME_DIR)),
                None => {
                    return Err(Error::new(ErrorType::Other(
                        "Home cannot be determined, use '--home'".to_owned(),
                    )))
                }
            },
        }?;

        Ok(home)
    }
}

async fn get_object(
    Path(path): Path<String>,
    State(odb): State<Arc<ObjectDB>>,
) -> (StatusCode, Vec<u8>) {
    let oid = match ObjectID::new_from_hex(&path) {
        Ok(oid) => oid,
        Err(error) => {
            error!("Object ID failed to parse: {error}");
            return (StatusCode::NOT_ACCEPTABLE, Vec::new());
        }
    };

    let object = match odb.try_read(&oid) {
        Ok(object) => object,
        Err(error) => {
            error!("Failed to get object: {error}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Vec::new());
        }
    };

    match object {
        None => (StatusCode::NOT_FOUND, Vec::new()),
        Some(mut d) => {
            let mut all = Vec::new();
            match d.read_to_end(&mut all) {
                Ok(_) => (StatusCode::OK, all),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Vec::new()),
            }
        }
    }
}
