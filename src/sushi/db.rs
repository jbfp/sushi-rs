use super::domain::*;
use super::*;
use rusqlite::{
    named_params,
    types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef},
    Connection, OptionalExtension, Result as RusqliteResult, NO_PARAMS,
};
use serde_json::Value;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FormatResult};

type Result<T> = std::result::Result<T, Error>;
type FromSqlResult<T> = std::result::Result<T, FromSqlError>;

#[derive(Debug)]
pub enum Error {
    RusqliteError(rusqlite::Error),
    GameVersionConflict,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        match self {
            Error::RusqliteError(e) => e.fmt(f),
            Error::GameVersionConflict => write!(f, "Game version conflict"),
        }
    }
}

impl StdError for Error {}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Error {
        Error::RusqliteError(error)
    }
}

#[derive(Clone)]
pub struct Database {
    path: String,
}

impl Database {
    pub fn new<S: ToString>(path: S) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    pub fn migrate(&self) -> Result<()> {
        let connection = self.open_connection()?;

        let user_version = connection.query_row(
            "SELECT user_version FROM pragma_user_version;",
            NO_PARAMS,
            |row| row.get(0),
        )?;

        info!("db user_version is {}", user_version);

        let mut new_user_version = user_version;

        if user_version < 1 {
            connection.execute_batch(
                "BEGIN;

                CREATE TABLE games
                ( id INTEGER PRIMARY KEY
                , data TEXT NOT NULL
                , players TEXT NOT NULL DEFAULT (json_array())
                , version INTEGER NOT NULL DEFAULT 0
                , created DATE NOT NULL DEFAULT (datetime('now'))
                , updated DATE NOT NULL DEFAULT (datetime('now'))
                );

                CREATE TRIGGER games_timestamp
                AFTER UPDATE OF version
                ON games
                BEGIN
                    UPDATE games
                    SET updated = datetime('now')
                    WHERE id = new.id;
                END;

                CREATE TABLE users
                ( id INTEGER PRIMARY KEY
                , name TEXT NOT NULL UNIQUE
                );

                CREATE TABLE games_users
                ( id INTEGER PRIMARY KEY
                , game_id INTEGER NOT NULL REFERENCES games (id)
                , user_id INTEGER NOT NULL REFERENCES users (id)
                );

                CREATE TRIGGER games_update_players
                AFTER INSERT ON games_users
                BEGIN
                    UPDATE games
                    SET players = (
                        SELECT json_group_array(u.name)
                        FROM games_users gu
                        INNER JOIN users u ON gu.user_id = u.id
                        WHERE gu.game_id = new.game_id
                    )
                    WHERE id = new.game_id;
                END;

                CREATE INDEX games_users_game_id ON games_users (game_id);
                CREATE INDEX games_users_user_id ON games_users (user_id);

                COMMIT;",
            )?;

            new_user_version = 1;
        }

        // Add additional migrations here as necessary

        if new_user_version > user_version {
            connection.pragma_update(None, "user_version", &new_user_version)?;
            info!("new user_version is {}", new_user_version);
        } else {
            info!("no migration performed");
        }

        Ok(())
    }

    pub fn get_or_insert_user_id(&self, user_name: &str) -> Result<UserId> {
        static GET_SQL: &str = "
            SELECT id
            FROM users
            WHERE name = :name";

        static INSERT_SQL: &str = "
            INSERT OR IGNORE INTO users (name)
            VALUES (:name)";

        let connection = self.open_connection()?;
        let params = named_params! { ":name": user_name };
        let value = connection
            .query_row_named(GET_SQL, params, |row| row.get(0))
            .optional()?;

        let user_id = match value {
            Some(user_id) => user_id,
            None => {
                info!("Inserting user {}", user_name);
                let params = named_params! { ":name": user_name };
                connection.execute_named(INSERT_SQL, params)?;
                UserId(connection.last_insert_rowid())
            }
        };

        Ok(user_id)
    }

    pub fn get_games_for_user(&self, user_id: UserId) -> Result<Vec<GameListItem>> {
        static SQL: &str = "
            SELECT g.id, g.players, g.updated
            FROM games g
            INNER JOIN games_users gu ON gu.game_id = g.id
            WHERE gu.user_id = :user_id
            ORDER BY g.updated DESC";

        let connection = self.open_connection()?;
        let mut statement = connection.prepare(SQL)?;
        let params = named_params! { ":user_id": user_id };
        let rows = statement.query_map_named(params, |row| {
            let id = row.get(0)?;
            let players = serde_json::from_value(row.get(1)?).unwrap();
            Ok(GameListItem { id, players })
        })?;

        let mut games = vec![];

        for row in rows {
            games.push(row?);
        }

        Ok(games)
    }

    pub fn persist_game(&self, game: &Game) -> Result<GameId> {
        let json = serde_json::to_string(&game).expect("can serialize game to json");

        let mut connection = self.open_connection()?;
        let tx = connection.transaction()?;

        let game_id = {
            static SQL: &str = "
                INSERT INTO games (data)
                VALUES (:data)";

            let mut statement = tx.prepare(SQL)?;

            statement.execute_named(named_params! {
                ":data": json
            })?;

            GameId(tx.last_insert_rowid())
        };

        {
            static SQL: &str = "
                INSERT INTO games_users (game_id, user_id)
                VALUES (:game_id, :user_id)";

            let mut statement = tx.prepare(SQL)?;

            for user_id in game.user_ids() {
                statement.execute_named(named_params! {
                    ":game_id": game_id,
                    ":user_id": user_id,
                })?;
            }
        }

        tx.commit()?;

        Ok(game_id)
    }

    pub fn read_game_for_user(
        &self,
        game_id: GameId,
        user_id: UserId,
    ) -> Result<Option<(Game, u8)>> {
        static SQL: &str = "
            SELECT data, version
            FROM games g
            INNER JOIN games_users gu ON g.id = gu.game_id
            WHERE gu.game_id = :game_id AND gu.user_id = :user_id";

        info!("Finding game {} for user {}", game_id, user_id);

        let params = named_params! {
            ":game_id": game_id,
            ":user_id": user_id,
        };

        let result = self
            .open_connection()?
            .prepare(SQL)?
            .query_row_named(params, |row| {
                let json: Value = row.get(0)?;
                let version = row.get(1)?;
                let game =
                    serde_json::from_value(json).expect("failed to deserialize json from db");
                Ok((game, version))
            })
            .optional()?;

        Ok(result)
    }

    pub fn update_game(&self, game_id: GameId, game: &Game, expected_version: u8) -> Result<()> {
        static SQL: &str = "
            UPDATE games
            SET data = :data
              , version = :new_version
            WHERE id = :id
            AND version = :expected_version";

        info!("Updating game {} at version {}", game_id, expected_version);

        let json = serde_json::to_string(&game).expect("can serialize game to json");
        let new_version = expected_version + 1;

        let updated = self
            .open_connection()?
            .prepare(SQL)?
            .execute_named(named_params! {
                ":id": game_id,
                ":data": json,
                ":expected_version": &expected_version,
                ":new_version": &new_version,
            })?;

        match updated {
            0 => Err(Error::GameVersionConflict),
            _ => Ok(()),
        }
    }

    pub fn read_game(&self, game_id: GameId) -> Result<Option<(Game, u8)>> {
        static SQL: &str = "
            SELECT data, version
            FROM games
            WHERE id = :id";

        info!("Finding game {} for system", game_id);

        let params = named_params! {
            ":id": game_id
        };

        let result = self
            .open_connection()?
            .prepare(SQL)?
            .query_row_named(params, |row| {
                let json: Value = row.get(0)?;
                let version = row.get(1)?;
                let game =
                    serde_json::from_value(json).expect("failed to deserialize json from db");
                Ok((game, version))
            })
            .optional()?;

        Ok(result)
    }

    fn open_connection(&self) -> Result<Connection> {
        let connection = Connection::open(&self.path)?;
        connection.pragma_update(None, "journal_mode", &"WAL")?;
        Ok(connection)
    }
}

impl ToSql for GameId {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput> {
        self.0.to_sql()
    }
}

impl FromSql for GameId {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        Ok(Self(i64::column_result(value)?))
    }
}

impl ToSql for UserId {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput> {
        self.0.to_sql()
    }
}

impl FromSql for UserId {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        Ok(Self(i64::column_result(value)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linked_hash_set::LinkedHashSet;
    use rusqlite::params;
    use serial_test::serial;
    use std::convert::TryFrom;
    use std::iter::FromIterator;
    use std::ops::Deref;

    struct Db {
        db: Database,
        connection: Connection,
    }

    impl Deref for Db {
        type Target = Database;

        fn deref(&self) -> &Self::Target {
            &self.db
        }
    }

    lazy_static! {
        static ref PLAYER1: UserId = UserId(1);
        static ref PLAYER2: UserId = UserId(2);
        static ref PLAYER3: UserId = UserId(3);
    }

    fn in_memory() -> Db {
        let db = Database::new("file::memory:?cache=shared");
        let connection = db.open_connection().unwrap();
        Db { db, connection }
    }

    fn setup_db() -> Db {
        let db = in_memory();
        db.migrate().unwrap();
        db
    }

    fn game() -> Game {
        Game::try_from(LinkedHashSet::from_iter(vec![*PLAYER1, *PLAYER2, *PLAYER3])).unwrap()
    }

    fn other_game() -> Game {
        Game::try_from(LinkedHashSet::from_iter(vec![*PLAYER2, *PLAYER3])).unwrap()
    }

    #[test]
    #[serial]
    fn can_migrate() {
        // arrange
        let db = in_memory();

        // act
        let result = db.migrate();

        // assert
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn get_or_insert_user_id_will_insert() {
        // arrange
        let db = setup_db();
        let user_name = "test";

        // act
        let result = db.get_or_insert_user_id(user_name);

        // assert
        let user_id = result.unwrap();

        let count: u32 = db
            .connection
            .query_row(
                "SELECT COUNT(*) FROM users WHERE id = ?1",
                params![user_id],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    #[serial]
    fn get_or_insert_user_id_will_get() {
        // arrange
        let db = setup_db();
        let user_name = "test";
        let expected = db.get_or_insert_user_id(user_name);

        // act
        let actual = db.get_or_insert_user_id(user_name);

        // assert
        assert!(expected.is_ok());
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), expected.unwrap());
    }

    #[test]
    #[serial]
    fn can_insert_game() {
        // arrange
        let db = setup_db();
        let game = game();

        // act
        let game_id = db.persist_game(&game).unwrap();

        // assert
        let count: u32 = db
            .connection
            .query_row(
                "SELECT COUNT(*) FROM games WHERE id = ?1",
                params![game_id],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    #[serial]
    fn can_read_game() {
        // arrange
        let db = setup_db();
        let game = game();
        let game_id = db.persist_game(&game).unwrap();

        // act
        let actual = db.read_game(game_id).unwrap();

        // assert
        assert_eq!(actual, Some((game, 0)));
    }

    #[test]
    #[serial]
    fn can_read_game_for_player() {
        // arrange
        let db = setup_db();
        let game = game();
        let game_id = db.persist_game(&game).unwrap();
        let user_id = *PLAYER1;

        // act
        let actual = db.read_game_for_user(game_id, user_id).unwrap();

        // assert
        assert_eq!(actual, Some((game, 0)));
    }

    #[test]
    #[serial]
    fn cannot_read_game_if_not_playing() {
        // arrange
        let db = setup_db();
        let game = game();
        let game_id = db.persist_game(&game).unwrap();
        let user_id = UserId(42);

        // act
        let actual = db.read_game_for_user(game_id, user_id).unwrap();

        // assert
        assert!(actual.is_none());
    }

    #[test]
    #[serial]
    fn can_get_games_for_user() {
        // arrange
        let db = setup_db();
        let num_games = 3;
        let user_id = *PLAYER1;

        for _ in 0..num_games {
            db.persist_game(&game()).unwrap();
        }

        db.persist_game(&other_game()).unwrap(); // Game without player 1

        // act
        let games = db.get_games_for_user(user_id).unwrap();

        // assert
        assert_eq!(games.len(), num_games);
    }

    #[test]
    #[serial]
    fn can_update_game() {
        // arrange
        let db = setup_db();
        let game = game();
        let game_id = db.persist_game(&game).unwrap();

        // act
        let actual = db.update_game(game_id, &game, 0);

        // assert
        assert!(actual.is_ok());
    }

    #[test]
    #[serial]
    fn update_increments_version() {
        // arrange
        let db = setup_db();
        let game = game();
        let game_id = db.persist_game(&game).unwrap();
        db.update_game(game_id, &game, 0).unwrap();

        // act
        let actual = db.read_game(game_id).unwrap();

        // assert
        assert_eq!(actual, Some((game, 1)));
    }

    #[test]
    #[serial]
    fn cannot_update_if_expected_version_does_not_match() {
        // arrange
        let db = setup_db();
        let game = game();
        let game_id = db.persist_game(&game).unwrap();
        db.update_game(game_id, &game, 0).unwrap();

        // act
        let error = db.update_game(game_id, &game, 0).unwrap_err();

        // assert
        assert!(match error {
            Error::GameVersionConflict => true,
            Error::RusqliteError(_) => false,
        });
    }
}
