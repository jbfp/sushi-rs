use super::types::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Error as FormatError, Formatter};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    face_up_cards: Vec<FaceUpCard>,
    hand: Hand,
    id: UserId,
    num_points: usize,
    num_puddings: usize,
    selected_cards: Vec<usize>,
}

impl Player {
    pub fn new(id: UserId, hand: Hand) -> Self {
        Self {
            face_up_cards: Vec::new(),
            hand,
            id,
            num_points: 0,
            num_puddings: 0,
            selected_cards: Vec::new(),
        }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn select_cards(&mut self, cards: Vec<usize>) -> Result<Vec<GameEvent>, SelectCardsError> {
        let mut events = vec![];

        let num_cards = cards.len();

        if num_cards > 2 {
            return Err(SelectCardsError::TooManyCards);
        }

        if num_cards > 1 {
            // Verify player has chopsticks
            let has_chopsticks = self.face_up_cards.iter().any(|fuc| match fuc {
                FaceUpCard::Card {
                    card: Card::Chopsticks,
                    ..
                } => true,
                _ => false,
            });

            if !has_chopsticks {
                return Err(SelectCardsError::NoChopsticks);
            }
        }

        for card in &cards {
            if !self.hand.contains_key(card) {
                return Err(SelectCardsError::DoesNotHaveCard);
            }
        }

        if !self.selected_cards.eq(&cards) {
            self.selected_cards = cards;
            events.push(GameEvent::CardsSelected(self.id));
        }

        Ok(events)
    }

    pub fn is_ready(&self) -> bool {
        !self.selected_cards.is_empty()
    }

    pub fn is_done(&self) -> bool {
        self.hand.is_empty()
    }

    pub fn step(&mut self) {
        // Remove cards to play from player's hand
        let played_cards = {
            let mut selected_cards = self.selected_cards.drain(..).collect::<Vec<_>>();

            selected_cards
                .drain(..)
                .map(|id| self.hand.remove_entry(&id).unwrap())
                .collect::<Vec<_>>()
        };

        // Use chopsticks if more than one card is played
        if played_cards.len() > 1 {
            // Remove chopsticks from face-up cards...
            let index = self
                .face_up_cards
                .iter()
                .position(|face_up_card| match face_up_card {
                    FaceUpCard::Card {
                        card: Card::Chopsticks,
                        ..
                    } => true,
                    _ => false,
                })
                .unwrap();

            match self.face_up_cards.remove(index) {
                FaceUpCard::Card { id, card } => {
                    // ... and add it back to the hand
                    self.hand.insert(id, card);
                }
                _ => unreachable!(),
            };
        }

        // Play the cards:
        for (id, card) in played_cards {
            if let Card::Pudding = card {
                self.num_puddings += 1;
            }

            if let Card::Nigiri(nigiri) = card {
                // Remove wasabi
                let index = self
                    .face_up_cards
                    .iter()
                    .position(|face_up_card| match face_up_card {
                        FaceUpCard::Card {
                            card: Card::Wasabi, ..
                        } => true,
                        _ => false,
                    });

                match index {
                    None => {
                        // Player does not have wasabi, add Nigir as normal card
                        self.face_up_cards.push(FaceUpCard::Card { id, card });
                    }

                    Some(index) => {
                        // Combine the wasabi and the nigiri
                        self.face_up_cards.remove(index);
                        self.face_up_cards.push(FaceUpCard::Wasabi { nigiri });
                    }
                }
            } else {
                self.face_up_cards.push(FaceUpCard::Card { id, card });
            }
        }
    }

    pub fn take_hand(&mut self) -> Hand {
        std::mem::replace(&mut self.hand, Hand::new())
    }

    pub fn give_hand(&mut self, hand: Hand) {
        self.hand = hand;
    }

    pub fn take_face_up_cards(&mut self) -> Vec<FaceUpCard> {
        std::mem::replace(&mut self.face_up_cards, Vec::new())
    }

    pub fn add_points(&mut self, score: usize) {
        self.num_points += score;
    }
}

impl PartialOrd for Player {
    fn partial_cmp(&self, rhs: &Player) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for Player {
    fn cmp(&self, rhs: &Player) -> Ordering {
        self.num_points
            .cmp(&rhs.num_points)
            .then(self.num_puddings.cmp(&rhs.num_puddings))
    }
}

impl Into<PlayerView> for Player {
    fn into(self) -> PlayerView {
        PlayerView {
            face_up_cards: self.face_up_cards,
            hand: self.hand,
            num_points: self.num_points,
            num_puddings: self.num_puddings,
            selected_cards: self.selected_cards,
        }
    }
}

impl Into<OpponentView> for Player {
    fn into(self) -> OpponentView {
        let ready = self.is_ready();

        OpponentView {
            face_up_cards: self.face_up_cards,
            id: self.id,
            num_cards: self.hand.len(),
            num_points: self.num_points,
            num_puddings: self.num_puddings,
            ready,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectCardsError {
    TooManyCards,
    NoChopsticks,
    DoesNotHaveCard,
}

impl Display for SelectCardsError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        use SelectCardsError::*;

        match self {
            TooManyCards => write!(f, "Too many cards selected"),
            NoChopsticks => write!(f, "Only one card can be played without chopsticks"),
            DoesNotHaveCard => write!(f, "Player does not have card"),
        }
    }
}

impl Error for SelectCardsError {}
