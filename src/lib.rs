#[cfg(feature = "async")]
mod async_elo;

mod elo;

pub use crate::elo::{Elo, EloStorage};

#[cfg(feature = "async")]
pub use async_elo::{AsyncElo, AsyncEloStorage};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Player {
    name: String,
    rating: usize,
    number_of_games: usize,
}

impl Player {
    pub fn new(name: String, rating: usize, number_of_games: usize) -> Self {
        Self {
            name,
            rating,
            number_of_games,
        }
    }

    pub fn rating(&self) -> usize {
        self.rating
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn number_of_games(&self) -> usize {
        self.number_of_games
    }
}

impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Player) -> Option<std::cmp::Ordering> {
        Some(match self.rating.cmp(&other.rating).reverse() {
            std::cmp::Ordering::Equal => {
                match self.number_of_games.cmp(&other.number_of_games).reverse() {
                    std::cmp::Ordering::Equal => self.name.cmp(&other.name),
                    ord => ord,
                }
            }
            ord => ord,
        })
    }
}

/// finds new ratings for two players
/// https://en.wikipedia.org/wiki/Elo_rating_system#Mathematical_details
pub fn update_rating(w: &Player, l: &Player, is_draw: bool) -> (usize, usize) {
    let winner_expected =
        1.0 / (1.0 + 10.0_f64.powf((l.rating as isize - w.rating as isize) as f64 / 400.0));
    let loser_expected =
        1.0 / (1.0 + 10.0_f64.powf((w.rating as isize - l.rating as isize) as f64 / 400.0));

    let factor = if is_draw { 0.5 } else { 0.0 };

    let winner_new_rating = w.rating() as f64 + 32.0 * (1.0 - factor - winner_expected);
    let loser_new_rating = l.rating() as f64 + 32.0 * (factor - loser_expected);

    (
        winner_new_rating.round() as usize,
        loser_new_rating.round() as usize,
    )
}

impl Ord for Player {
    fn cmp(&self, other: &Player) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
