#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

mod sushi;

use actix_files::{Files, NamedFile};
use actix_web::{middleware::Logger, web, App, HttpServer, Result as ActixResult};
use listenfd::ListenFd;
use std::io::Error as IoError;
use tokio::sync::mpsc::unbounded_channel;

#[actix_rt::main]
async fn main() -> Result<(), IoError> {
    pretty_env_logger::init();

    let db = sushi::Database::new("./sushi.db");

    db.migrate().expect("failed to migrate database");

    let (tx, rx) = unbounded_channel();

    // Create game event broadcaster
    let broadcaster = sushi::Broadcaster::new();

    // Start countdown listener
    tokio::spawn(sushi::receiver(broadcaster.clone(), db.clone(), rx));

    let mut listenfd = ListenFd::from_env();

    let mut server = HttpServer::new(move || {
        let db = db.clone();
        let broadcaster = broadcaster.clone();
        let tx = tx.clone();

        App::new()
            .wrap(Logger::default())
            .configure(|cfg| sushi::app(db, broadcaster, tx, cfg))
            .default_service(static_files())
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:8080")?
    };

    server.run().await
}

fn static_files() -> Files {
    Files::new("/", "./frontend/build")
        .index_file("index.html")
        .default_handler(web::get().to(index))
}

async fn index() -> ActixResult<NamedFile> {
    Ok(NamedFile::open("./frontend/build/index.html")?)
}
