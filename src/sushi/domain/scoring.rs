use super::types::*;
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;

pub fn score_round<K: Copy + Eq + Hash>(round: &HashMap<K, Vec<FaceUpCard>>) -> HashMap<K, usize> {
    let len = round.len();
    let mut num_points_by_key = HashMap::with_capacity(len);
    let mut num_maki_rolls_by_key = HashMap::with_capacity(len);

    for (key, face_up_cards) in round {
        let (num_points, num_maki_rolls) = score_face_up_cards(face_up_cards);
        num_points_by_key.insert(*key, num_points);
        num_maki_rolls_by_key.insert(*key, num_maki_rolls);
    }

    for (key, num_points) in score_maki_rolls(num_maki_rolls_by_key) {
        if let Some(n) = num_points_by_key.get_mut(&key) {
            *n += num_points;
        }
    }

    num_points_by_key
}

fn score_face_up_cards(face_up_cards: &[FaceUpCard]) -> (usize, usize) {
    let mut nigiri_points = 0;
    let mut num_dumplings = 0;
    let mut num_maki_rolls = 0;
    let mut num_sashimis = 0;
    let mut num_tempuras = 0;

    for face_up_card in face_up_cards {
        match face_up_card {
            FaceUpCard::Card { card, .. } => match card {
                Card::Dumpling => num_dumplings += 1,
                Card::MakiRolls(n) => num_maki_rolls += *n as usize,
                Card::Nigiri(nigiri) => nigiri_points += score_nigiri(nigiri),
                Card::Sashimi => num_sashimis += 1,
                Card::Tempura => num_tempuras += 1,
                _ => {}
            },

            FaceUpCard::Wasabi { nigiri } => {
                nigiri_points += 3 * score_nigiri(nigiri);
            }
        }
    }

    let dumpling_points = score_dumplings(num_dumplings);
    let sashimi_points = score_sashimis(num_sashimis);
    let tempura_points = score_tempuras(num_tempuras);
    let total_points = dumpling_points + nigiri_points + sashimi_points + tempura_points;
    (total_points, num_maki_rolls)
}

/// Scores a Nigiri card:
/// - Egg = 1 point
/// - Salmon = 2 points
/// - Squid = 3 points
fn score_nigiri(nigiri: &Nigiri) -> usize {
    *nigiri as usize
}

/// Calculates the score of the given set of Dumplings:
/// - 0 dumplings = 0 points
/// - 1 dumpling = 1 point
/// - 2 dumplings = 3 points
/// - 3 dumplings = 6 points
/// - 4 dumplings = 10 points
/// - 5 or more dumplings = 15 points
fn score_dumplings(num_dumplings: usize) -> usize {
    match num_dumplings {
        0 => 0,
        1 => 1,
        2 => 3,
        3 => 6,
        4 => 10,
        _ => 15,
    }
}

/// Calculates the score of the given set of Sashimis.
/// The score is 10 points per triplet of Sashimis.
fn score_sashimis(num_sashimis: usize) -> usize {
    10 * (num_sashimis / 3)
}

/// Calculates the score of the given set of Tempuras.
/// The score is 5 points per pair of Tempuras.
fn score_tempuras(num_tempuras: usize) -> usize {
    5 * (num_tempuras / 2)
}

/// Scores a set of Maki Rolls, each assigned to a Player ID.
/// The player with the most Maki Rolls scores 6 points. If multiple players tie
/// for the most, they split the 6 points evenly
/// (ignoring any remainder) and no second place points are awarded.
///
/// The player with the second most Maki Rolls scores 3 points. If multiple players
/// tie for second place, they split the points evenly (ignoring any remainder).
fn score_maki_rolls<K: Eq + Hash>(num_maki_rolls_by_key: HashMap<K, usize>) -> HashMap<K, usize> {
    const FIRST_PLACE_POINTS: usize = 6;
    const SECOND_PLACE_POINTS: usize = 3;

    // Resolve ties:
    let mut maki_rolls = num_maki_rolls_by_key
        .values()
        .copied()
        .collect::<BinaryHeap<_>>();

    let first = maki_rolls.pop().unwrap_or_default();
    let second = maki_rolls.pop().unwrap_or_default();
    let tie = first == second;

    // Determine how many points first place will get
    // - if no-one has any rolls, no one gets any points.
    // - if there is a tie for first place, evenly divide the points
    // - otherwise first place gets full points
    let first_place_points = if first == 0 {
        0
    } else if tie {
        // Add 2 to the divisor since we popped both first and second place off earlier
        // and there may be more with the same amount of rolls
        FIRST_PLACE_POINTS / (2 + maki_rolls.iter().filter(|&&n| n == first).count())
    } else {
        FIRST_PLACE_POINTS
    };

    // Determine how many points second place will get
    // - if no-one has any rolls after first place, no one gets any points.
    // - if there is a tie for first place, no one gets second place points
    // - if there is a tie for second place, evenly divide the points
    // - otherwise second place gets full points
    let second_place_points = if second == 0 || tie {
        0
    } else {
        // Add 1 to the divisor since we popped second place off earlier
        // and there may be more with the same amount of rolls
        SECOND_PLACE_POINTS / (1 + maki_rolls.iter().filter(|&&n| n == second).count())
    };

    // Award points:
    let mut results = HashMap::new();

    for (key, num_maki_rolls) in num_maki_rolls_by_key {
        let num_points = if num_maki_rolls == first {
            first_place_points
        } else if num_maki_rolls == second {
            second_place_points
        } else {
            0
        };

        results.insert(key, num_points);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Nigiri::Egg => 1)]
    #[test_case(Nigiri::Salmon => 2)]
    #[test_case(Nigiri::Squid => 3)]
    fn score_nigiri_tests(nigiri: Nigiri) -> usize {
        score_nigiri(&nigiri)
    }

    #[test_case(0 => 0)]
    #[test_case(1 => 0)]
    #[test_case(2 => 5)]
    #[test_case(3 => 5)]
    #[test_case(4 => 10)]
    fn score_tempuras_tests(n: usize) -> usize {
        score_tempuras(n)
    }

    #[test_case(0 => 0)]
    #[test_case(1 => 0)]
    #[test_case(2 => 0)]
    #[test_case(3 => 10)]
    #[test_case(4 => 10)]
    #[test_case(5 => 10)]
    #[test_case(6 => 20)]
    #[test_case(7 => 20)]
    fn score_sashimis_tests(n: usize) -> usize {
        score_sashimis(n)
    }

    #[test_case(0 => 0)]
    #[test_case(1 => 1)]
    #[test_case(2 => 3)]
    #[test_case(3 => 6)]
    #[test_case(4 => 10)]
    #[test_case(5 => 15)]
    #[test_case(6 => 15)]
    fn score_dumplings_tests(n: usize) -> usize {
        score_dumplings(n)
    }

    #[test]
    fn score_maki_rolls_0_cannot_be_second_place() {
        // arrange
        let mut rolls = HashMap::new();
        rolls.insert(0, 1);
        rolls.insert(1, 0);

        // act
        let actual = score_maki_rolls(rolls);

        // assert
        assert_eq!(actual.get(&0), Some(&6));
        assert_eq!(actual.get(&1), Some(&0));
    }

    #[test]
    fn score_maki_rolls_first_second_third_places() {
        // arrange
        let mut rolls = HashMap::new();
        rolls.insert(0, 2);
        rolls.insert(1, 1);
        rolls.insert(2, 0);

        // act
        let actual = score_maki_rolls(rolls);

        // assert
        assert_eq!(actual.get(&0), Some(&6));
        assert_eq!(actual.get(&1), Some(&3));
        assert_eq!(actual.get(&2), Some(&0));
    }

    #[test]
    fn score_maki_rolls_first_place_twoway_tie() {
        // arrange
        let mut rolls = HashMap::new();
        rolls.insert(0, 2);
        rolls.insert(1, 2);
        rolls.insert(2, 1);
        rolls.insert(3, 0);

        // act
        let actual = score_maki_rolls(rolls);

        // assert
        assert_eq!(actual.get(&0), Some(&3));
        assert_eq!(actual.get(&1), Some(&3));
        assert_eq!(actual.get(&2), Some(&0));
        assert_eq!(actual.get(&3), Some(&0));
    }

    #[test]
    fn score_maki_rolls_first_place_threeway_tie() {
        // arrange
        let mut rolls = HashMap::new();
        rolls.insert(0, 2);
        rolls.insert(1, 2);
        rolls.insert(2, 2);
        rolls.insert(3, 1);
        rolls.insert(4, 0);

        // act
        let actual = score_maki_rolls(rolls);

        // assert
        assert_eq!(actual.get(&0), Some(&2));
        assert_eq!(actual.get(&1), Some(&2));
        assert_eq!(actual.get(&2), Some(&2));
        assert_eq!(actual.get(&3), Some(&0));
        assert_eq!(actual.get(&4), Some(&0));
    }

    #[test]
    fn score_maki_rolls_first_place_fourway_tie() {
        // arrange
        let mut rolls = HashMap::new();
        rolls.insert(0, 2);
        rolls.insert(1, 2);
        rolls.insert(2, 2);
        rolls.insert(3, 2);
        rolls.insert(4, 1);

        // act
        let actual = score_maki_rolls(rolls);

        // assert
        assert_eq!(actual.get(&0), Some(&1));
        assert_eq!(actual.get(&1), Some(&1));
        assert_eq!(actual.get(&2), Some(&1));
        assert_eq!(actual.get(&3), Some(&1));
        assert_eq!(actual.get(&4), Some(&0));
    }

    #[test]
    fn score_maki_rolls_first_place_fiveway_tie() {
        // arrange
        let mut rolls = HashMap::new();
        rolls.insert(0, 1);
        rolls.insert(1, 1);
        rolls.insert(2, 1);
        rolls.insert(3, 1);
        rolls.insert(4, 1);

        // act
        let actual = score_maki_rolls(rolls);

        // assert
        assert_eq!(actual.get(&0), Some(&1));
        assert_eq!(actual.get(&1), Some(&1));
        assert_eq!(actual.get(&2), Some(&1));
        assert_eq!(actual.get(&3), Some(&1));
        assert_eq!(actual.get(&4), Some(&1));
    }

    #[test]
    fn score_maki_rolls_second_place_twoway_tie() {
        // arrange
        let mut rolls = HashMap::new();
        rolls.insert(0, 2);
        rolls.insert(1, 1);
        rolls.insert(2, 1);
        rolls.insert(3, 0);

        // act
        let actual = score_maki_rolls(rolls);

        // assert
        assert_eq!(actual.get(&0), Some(&6));
        assert_eq!(actual.get(&1), Some(&1));
        assert_eq!(actual.get(&2), Some(&1));
        assert_eq!(actual.get(&3), Some(&0));
    }

    #[test]
    fn score_maki_rolls_second_place_threeway_tie() {
        // arrange
        let mut rolls = HashMap::new();
        rolls.insert(0, 2);
        rolls.insert(1, 1);
        rolls.insert(2, 1);
        rolls.insert(3, 1);
        rolls.insert(4, 0);

        // act
        let actual = score_maki_rolls(rolls);

        // assert
        assert_eq!(actual.get(&0), Some(&6));
        assert_eq!(actual.get(&1), Some(&1));
        assert_eq!(actual.get(&2), Some(&1));
        assert_eq!(actual.get(&3), Some(&1));
        assert_eq!(actual.get(&4), Some(&0));
    }

    #[test]
    fn score_maki_rolls_second_place_fourway_tie() {
        // arrange
        let mut rolls = HashMap::new();
        rolls.insert(0, 2);
        rolls.insert(1, 1);
        rolls.insert(2, 1);
        rolls.insert(3, 1);
        rolls.insert(4, 1);

        // act
        let actual = score_maki_rolls(rolls);

        // assert
        assert_eq!(actual.get(&0), Some(&6));
        assert_eq!(actual.get(&1), Some(&0));
        assert_eq!(actual.get(&2), Some(&0));
        assert_eq!(actual.get(&3), Some(&0));
        assert_eq!(actual.get(&4), Some(&0));
    }
}
