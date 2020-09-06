use super::player::*;
use super::scoring::*;
use super::types::*;
use linked_hash_set::LinkedHashSet;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Display, Error as FormatError, Formatter};
use std::iter::empty;
use std::iter::FromIterator;

const MIN_GAME_SIZE: usize = 2;
const MAX_GAME_SIZE: usize = 5;
const NUM_ROUNDS: usize = 3;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    round: usize,
    hands: Vec<Hand>,
    players: Vec<Player>,
    winner: Option<UserId>,
}

impl Game {
    pub fn user_ids(&self) -> Vec<UserId> {
        self.players.iter().map(|p| p.id()).collect()
    }

    pub fn select_cards(
        &mut self,
        user_id: UserId,
        cards: Vec<usize>,
    ) -> Result<Vec<GameEvent>, SelectCardsError> {
        self.players
            .iter_mut()
            .find(|p| p.id() == user_id)
            .expect("failed to find player in game")
            .select_cards(cards)
    }

    pub fn ready_to_end_turn(&self) -> bool {
        self.players.iter().all(|p| p.is_ready())
    }

    pub fn end_turn(&mut self) -> Vec<GameEvent> {
        let mut events = vec![];

        if self.ready_to_end_turn() {
            for player in &mut self.players {
                player.step();
            }

            events.push(GameEvent::TurnOver);

            if self.players.iter().all(|p| p.is_done()) {
                for event in self.end_round() {
                    events.push(event);
                }
            } else {
                let num_players = self.players.len();
                let mut hands = VecDeque::with_capacity(num_players);

                for player in &mut self.players {
                    let hand = player.take_hand();
                    hands.push_back(hand);
                }

                hands.rotate_right(1);

                for player in &mut self.players {
                    let hand = hands.pop_front().unwrap();
                    player.give_hand(hand);
                }
            }
        }

        events
    }

    fn end_round(&mut self) -> Vec<GameEvent> {
        let mut events = vec![];

        let round = self
            .players
            .iter_mut()
            .map(|p| (p.id(), p.take_face_up_cards()))
            .collect::<HashMap<_, _>>();

        let scores_by_user_id = score_round(&round);

        for player in &mut self.players {
            let id = player.id();

            if let Some(points) = scores_by_user_id.get(&id) {
                player.add_points(*points);
            }
        }

        events.push(GameEvent::RoundOver(RoundOver {
            round: self.round,
            points: scores_by_user_id,
        }));

        self.round += 1;

        if self.round > NUM_ROUNDS {
            for event in self.end_game() {
                events.push(event);
            }
        } else {
            for player in &mut self.players {
                let hand = self.hands.pop().expect("not enough hands");
                player.give_hand(hand);
            }
        }

        events
    }

    fn end_game(&mut self) -> Vec<GameEvent> {
        self.winner = BinaryHeap::from_iter(&self.players).peek().map(|p| p.id());

        self.winner
            .iter()
            .copied()
            .map(GameEvent::GameOver)
            .collect::<Vec<_>>()
    }

    pub fn into(self, user_id: UserId) -> GameView {
        let mut player_view = None;
        let mut opponent_views = Vec::with_capacity(self.players.len());

        for player in self.players {
            if player.id() == user_id {
                player_view = Some(player.into());
            } else {
                opponent_views.push(player.into());
            }
        }

        GameView {
            round: self.round,
            player: player_view,
            opponents: opponent_views,
            winner: self.winner,
        }
    }
}

impl TryFrom<LinkedHashSet<UserId>> for Game {
    type Error = CreateGameError;

    fn try_from(user_ids: LinkedHashSet<UserId>) -> Result<Self, Self::Error> {
        let num_players = user_ids.len();

        if num_players < MIN_GAME_SIZE {
            return Err(CreateGameError::TooFewPlayers(num_players));
        }

        if num_players > MAX_GAME_SIZE {
            return Err(CreateGameError::TooManyPlayers(num_players));
        }

        let mut cards = CARDS.clone();
        let mut rng = thread_rng();
        cards.shuffle(&mut rng);

        let num_cards_per_player = match num_players {
            2 => 5,
            3 => 9,
            4 => 8,
            5 => 7,
            _ => unreachable!(),
        };

        let num_hands = NUM_ROUNDS * num_players;
        let mut hands = Vec::with_capacity(num_hands);

        for _ in 0..num_hands {
            let mut hand = Hand::with_capacity(num_cards_per_player);

            for _ in 0..num_cards_per_player {
                let (id, card) = cards.pop().expect("not enough cards");
                hand.insert(id, card);
            }

            hands.push(hand);
        }

        let mut players = Vec::with_capacity(num_players);

        for user_id in user_ids {
            let hand = hands.pop().expect("not enough hands");
            let player = Player::new(user_id, hand);
            players.push(player);
        }

        Ok(Self {
            round: 1,
            hands,
            players,
            winner: None,
        })
    }
}

// Error types

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateGameError {
    TooFewPlayers(usize),
    TooManyPlayers(usize),
}

impl Display for CreateGameError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        use CreateGameError::*;

        match self {
            TooFewPlayers(1) => write!(f, "You must add at least 1 more player to the game."),

            TooManyPlayers(n) => {
                let to_remove = n - MAX_GAME_SIZE;
                let suffix = if to_remove == 1 { "" } else { "s" };

                write!(
                    f,
                    "Remove at least {} player{} from the game.",
                    to_remove, suffix
                )
            }

            _ => unreachable!(),
        }
    }
}

impl Error for CreateGameError {}

lazy_static! {
    static ref CARDS: Vec<(usize, Card)> = empty()
        .chain(vec![Card::Tempura].repeat(14))
        .chain(vec![Card::Sashimi].repeat(14))
        .chain(vec![Card::Dumpling].repeat(14))
        .chain(vec![Card::MakiRolls(MakiRolls::Two)].repeat(12))
        .chain(vec![Card::MakiRolls(MakiRolls::Three)].repeat(8))
        .chain(vec![Card::MakiRolls(MakiRolls::One)].repeat(6))
        .chain(vec![Card::Nigiri(Nigiri::Salmon)].repeat(10))
        .chain(vec![Card::Nigiri(Nigiri::Squid)].repeat(5))
        .chain(vec![Card::Nigiri(Nigiri::Egg)].repeat(5))
        .chain(vec![Card::Pudding].repeat(10))
        .chain(vec![Card::Wasabi].repeat(6))
        .chain(vec![Card::Chopsticks].repeat(4))
        .enumerate()
        .collect();
}
