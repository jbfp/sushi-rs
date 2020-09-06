use super::db::{Error as DbError, *};
use super::domain::*;
use super::pubsub::*;
use super::GameId;
use actix_web::{
    dev::HttpResponseBuilder,
    error, get,
    http::StatusCode,
    post, put,
    web::{Data, HttpResponse, Json, Path, ServiceConfig},
    HttpRequest, ResponseError, Result as ActixResult,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use linked_hash_set::LinkedHashSet;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::Display;
use std::time::Duration;
use tokio::{stream::StreamExt, sync::mpsc::UnboundedSender};

const JWT_ALGORITHM: Algorithm = Algorithm::HS256;

lazy_static! {
    static ref ENCODING_KEY: EncodingKey = EncodingKey::from_secret(b"secret");
    static ref DECODING_KEY: DecodingKey<'static> = DecodingKey::from_secret(b"secret");
}

type AppResult = ActixResult<HttpResponse>;

pub fn app(
    db: Database,
    broadcaster: Broadcaster,
    tx: UnboundedSender<Countdown>,
    cfg: &mut ServiceConfig,
) {
    cfg.data(db)
        .data(broadcaster)
        .data(tx)
        .service(login)
        .service(get_games)
        .service(create_game)
        .service(get_game)
        .service(select_cards)
        .service(stream);
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: UserId,
    pub name: String,
}

#[post("/api/login")]
async fn login(db: Data<Database>, user_name: Json<String>) -> AppResult {
    let user_id = db.get_or_insert_user_id(&user_name)?;

    info!("Log in for {}; id is {}", user_name, user_id);

    let jwt = jsonwebtoken::encode(
        &Header::new(JWT_ALGORITHM),
        &Claims {
            sub: user_id,
            name: user_name.0,
        },
        &ENCODING_KEY,
    )
    .expect("failed to encode jwt");

    ok(jwt)
}

#[get("/api/games")]
async fn get_games(db: Data<Database>, request: HttpRequest) -> AppResult {
    let user_id = extract_user_id(&request)?;
    info!("Getting games for {}", user_id);
    let game_ids = db.get_games_for_user(user_id)?;
    ok(game_ids)
}

#[post("/api/games")]
async fn create_game(
    db: Data<Database>,
    request: HttpRequest,
    opponents: Json<Vec<String>>,
) -> AppResult {
    let user_id = extract_user_id(&request)?;
    let mut user_ids = LinkedHashSet::new();
    user_ids.insert(user_id);

    for opponent in &opponents.0 {
        let trimmed = opponent.trim();

        if trimmed.is_empty() {
            return Ok(failure("Missing opponent name"));
        }

        let user_id = db.get_or_insert_user_id(trimmed)?;

        if !user_ids.insert(user_id) {
            return Ok(failure(format!(
                "Player '{}' is already in the game",
                trimmed
            )));
        }
    }

    match Game::try_from(user_ids) {
        Err(e) => Ok(failure(e)),

        Ok(game) => {
            let game_id = db.persist_game(&game)?;
            Ok(success(game_id))
        }
    }
}

#[get("/api/games/{game_id}")]
async fn get_game(db: Data<Database>, request: HttpRequest, game_id: Path<GameId>) -> AppResult {
    let user_id = extract_user_id(&request)?;
    let (game, _) = get_game_for_user(&db, *game_id, user_id)?;
    let view = game.into(user_id);
    ok(view)
}

#[put("/api/games/{game_id}")]
async fn select_cards(
    db: Data<Database>,
    countdown_tx: Data<UnboundedSender<Countdown>>,
    broadcaster: Data<Broadcaster>,
    game_id: Path<GameId>,
    selected_cards: Json<Vec<usize>>,
    request: HttpRequest,
) -> AppResult {
    let user_id = extract_user_id(&request)?;
    let game_id = *game_id;
    let (mut game, version) = get_game_for_user(&db, game_id, user_id)?;

    info!("Found game at version {}", version);

    match game.select_cards(user_id, selected_cards.0) {
        Err(e) => Ok(failure(e)),
        Ok(events) => {
            if !events.is_empty() {
                countdown_tx
                    .send(Countdown::Cancel { game_id })
                    .expect("failed to send countdown cancel event");

                db.update_game(game_id, &game, version)?;

                if game.ready_to_end_turn() {
                    const COUNTDOWN_MS: u64 = 3000;

                    countdown_tx
                        .send(Countdown::Start {
                            game_id,
                            duration: Duration::from_millis(COUNTDOWN_MS),
                        })
                        .expect("failed to send countdown started event");
                }

                for event in &events {
                    broadcaster.send(game_id, event).await;
                }
            }

            Ok(success(()))
        }
    }
}

#[get("/api/games/{game_id}/stream")]
async fn stream(game_id: Path<GameId>, broadcaster: Data<Broadcaster>) -> HttpResponse {
    let rx = broadcaster
        .subscribe(*game_id)
        .await
        .into_stream()
        .map(|r| r.map_err(|_| error::ErrorInternalServerError("")));

    HttpResponseBuilder::new(StatusCode::OK)
        .content_type("text/event-stream")
        .header("Cache-Control", "no-transform")
        .keep_alive()
        .no_chunking()
        .streaming(rx)
}

fn extract_user_id(request: &HttpRequest) -> ActixResult<UserId> {
    let header = request
        .headers()
        .get("Authorization")
        .ok_or_else(|| error::ErrorUnauthorized(""))?
        .to_str()
        .map_err(error::ErrorBadRequest)?;

    let parts = header.split_whitespace().collect::<Vec<_>>();

    match parts.as_slice() {
        ["Bearer", jwt] => {
            let mut validation = Validation::new(JWT_ALGORITHM);
            validation.validate_exp = false;

            jsonwebtoken::decode::<Claims>(jwt, &DECODING_KEY, &validation)
                .map(|c| c.claims.sub)
                .map_err(error::ErrorUnauthorized)
        }

        _ => Err(error::ErrorUnauthorized("")),
    }
}

fn get_game_for_user(db: &Database, game_id: GameId, user_id: UserId) -> ActixResult<(Game, u8)> {
    db.read_game_for_user(game_id, user_id)?
        .ok_or_else(|| error::ErrorNotFound(""))
}

fn ok<T: Serialize>(payload: T) -> AppResult {
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(payload))
}

#[derive(Serialize)]
struct ResponseBody<T> {
    success: bool,
    payload: Option<T>,
    error: Option<String>,
}

fn success<T: Serialize>(payload: T) -> HttpResponse {
    HttpResponseBuilder::new(StatusCode::OK).json(ResponseBody {
        success: true,
        payload: Some(payload),
        error: None,
    })
}

fn failure<E: Display>(error: E) -> HttpResponse {
    HttpResponseBuilder::new(StatusCode::OK).json(ResponseBody {
        success: false,
        payload: Option::<()>::None,
        error: Some(error.to_string()),
    })
}

impl ResponseError for DbError {
    fn status_code(&self) -> StatusCode {
        match self {
            DbError::RusqliteError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DbError::GameVersionConflict => StatusCode::CONFLICT,
        }
    }
}
