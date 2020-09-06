use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Error as FormatError, Formatter};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "makiRolls")]
pub enum MakiRolls {
    One = 1,
    Two = 2,
    Three = 3,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "nigiri")]
pub enum Nigiri {
    Egg = 1,
    Salmon = 2,
    Squid = 3,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Card {
    Chopsticks,
    Dumpling,
    MakiRolls(MakiRolls),
    Nigiri(Nigiri),
    Pudding,
    Sashimi,
    Tempura,
    Wasabi,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum FaceUpCard {
    Card { id: usize, card: Card },
    Wasabi { nigiri: Nigiri },
}

pub type Hand = HashMap<usize, Card>;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct UserId(pub i64);

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundOver {
    pub round: usize,
    pub points: HashMap<UserId, usize>,
}

#[derive(Clone, Debug)]
pub enum GameEvent {
    CardsSelected(UserId),
    CountdownStarted(Duration),
    CountdownCancelled,
    TurnOver,
    RoundOver(RoundOver),
    GameOver(UserId),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    pub round: usize,
    pub player: Option<PlayerView>,
    pub opponents: Vec<OpponentView>,
    pub winner: Option<UserId>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerView {
    pub face_up_cards: Vec<FaceUpCard>,
    pub hand: HashMap<usize, Card>,
    pub num_points: usize,
    pub num_puddings: usize,
    pub selected_cards: Vec<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpponentView {
    pub face_up_cards: Vec<FaceUpCard>,
    pub id: UserId,
    pub num_cards: usize,
    pub num_points: usize,
    pub num_puddings: usize,
    pub ready: bool,
}
