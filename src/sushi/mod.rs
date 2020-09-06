mod db;
mod domain;
mod handlers;
mod pubsub;

pub use db::Database;
pub use handlers::app;
pub use pubsub::{receiver, Broadcaster};

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error as FormatError, Formatter};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct GameId(pub i64);

impl Display for GameId {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        self.0.fmt(f)
    }
}

#[derive(Debug, Serialize)]
pub struct GameListItem {
    id: GameId,
    players: Vec<String>,
}
