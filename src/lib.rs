use std::{
    collections::{hash_map::Iter, HashMap},
    ops::{Index, IndexMut},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Player {
    name: String,
    rating: usize,
    numer_of_games: usize,
}

impl Player {
    pub fn rating(&self) -> usize {
        self.rating
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn numer_of_games(&self) -> usize {
        self.numer_of_games
    }
}

impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Player) -> Option<std::cmp::Ordering> {
        Some(match self.rating.cmp(&other.rating).reverse() {
            std::cmp::Ordering::Equal => {
                match self.numer_of_games.cmp(&other.numer_of_games).reverse() {
                    std::cmp::Ordering::Equal => self.name.cmp(&other.name),
                    ord => ord,
                }
            }
            ord => ord,
        })
    }
}

impl Ord for Player {
    fn cmp(&self, other: &Player) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug)]
pub struct Elo {
    players: HashMap<String, Player>,
    starting_elo: usize,
}

impl Elo {
    pub fn new() -> Elo {
        Elo {
            players: HashMap::new(),
            starting_elo: 1000,
        }
    }

    pub fn add_player<S: ToString>(&mut self, name: S) {
        self.players.insert(
            name.to_string(),
            Player {
                name: name.to_string(),
                rating: self.starting_elo,
                numer_of_games: 0,
            },
        );
    }

    pub fn try_add(&mut self, name: &str) {
        if !self.players.contains_key(name) {
            self.add_player(name);
        }
    }

    /// If is_draw is true, the game is a draw.
    /// If is_draw is false, the game is won by the first player.
    pub fn add_win(&mut self, player1: &str, player2: &str, is_draw: bool) -> Result<(), String> {
        if player1 == player2 {
            return Err(format!(
                "{} can't play against themselves (you friendless loser)",
                player1
            ));
        }

        self.try_add(player1);
        self.try_add(player2);
        let w = &self.players[player1];
        let l = &self.players[player2];

        let winner_expected =
            1.0 / (1.0 + 10.0_f64.powf((l.rating as isize - w.rating as isize) as f64 / 400.0));
        let loser_expected =
            1.0 / (1.0 + 10.0_f64.powf((w.rating as isize - l.rating as isize) as f64 / 400.0));

        let factor = if is_draw {
            0.5
        } else {
            0.0
        };

        let winner_new_rating = w.rating() as f64 + 32.0 * (1.0 - factor - winner_expected);
        let loser_new_rating = l.rating() as f64 + 32.0 * (factor - loser_expected);

        let mut update_rating = |player, new_rating: f64| {
            let p = self.players.get_mut(player).unwrap();
            p.rating = new_rating.round() as usize;
            p.numer_of_games += 1;
        };

        update_rating(player1, winner_new_rating);
        update_rating(player2, loser_new_rating);

        Ok(())
    }

    pub fn get_player(&self, name: &str) -> Option<&Player> {
        self.players.get(name)
    }
}

impl Default for Elo {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<&str> for Elo {
    type Output = Player;

    fn index(&self, name: &str) -> &Player {
        self.players.get(name).unwrap()
    }
}

impl IndexMut<&str> for Elo {
    fn index_mut(&mut self, name: &str) -> &mut Player {
        self.players.get_mut(name).unwrap()
    }
}

impl<'a> IntoIterator for &'a Elo {
    type Item = &'a Player;
    type IntoIter = EloIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        EloIter {
            elo: self.players.iter(),
        }
    }
}

pub struct EloIter<'a> {
    elo: Iter<'a, String, Player>,
}

impl<'a> Iterator for EloIter<'a> {
    type Item = &'a Player;

    fn next(&mut self) -> Option<Self::Item> {
        self.elo.next().map(|(_, player)| player)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_no_friends() {
        let mut elo = Elo::new();
        elo.add_player("a");

        assert!(elo.add_win("a", "a", false).is_err());
    }

    #[test]
    fn dual() {
        let mut elo = Elo::new();
        elo.add_player("a");
        elo.add_player("b");

        elo.add_win("a", "b", false).unwrap();
        elo.add_win("b", "a", false).unwrap();

        assert_eq!(elo["a"].rating(), 999);
        assert_eq!(elo["b"].rating(), 1001);

        assert_eq!(elo["a"].numer_of_games(), 2);
        assert_eq!(elo["b"].numer_of_games(), 2);
    }

    #[test]
    fn dual_draw() {
        let mut elo = Elo::new();
        elo.add_player("a");
        elo.add_player("b");

        elo.add_win("a", "b", true).unwrap();

        assert_eq!(elo["a"].rating(), 1000);
        assert_eq!(elo["b"].rating(), 1000);
    }

    #[test]
    fn ordering() {
        let mut elo = Elo::new();
        elo.add_player("a");
        elo.add_player("b");
        elo.add_player("c");
        elo.add_player("d");

        elo.add_win("a", "b", false).unwrap();
        elo.add_win("a", "b", false).unwrap();
        elo.add_win("a", "c", false).unwrap();

        // force b rating, to see ordering with comparison of c
        elo["b"].rating = 985;

        // add player d, see check that name is ordered lexicographically
        elo["d"].rating = 985;
        elo["d"].numer_of_games = 2;

        let mut players = elo.into_iter().collect::<Vec<_>>();
        players.sort();
        assert_eq!(players[0].name(), "a");
        assert_eq!(players[1].name(), "b");
        assert_eq!(players[2].name(), "d");
        assert_eq!(players[3].name(), "c");
    }
}
