use std::{
    collections::{hash_map::Iter, HashMap},
    ops::{Index, IndexMut},
};

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

    pub fn numer_of_games(&self) -> usize {
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

impl Ord for Player {
    fn cmp(&self, other: &Player) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub trait EloStorage<'a, I>
where
    I: Iterator<Item = &'a Player>,
{
    fn add_player(&mut self, player: Player);
    fn update_player(&mut self, player: &Player);
    fn get(&self, name: &str) -> Option<&Player>;
    fn get_mut(&mut self, name: &str) -> Option<&mut Player>;
    fn iter(&'a self) -> I;
}

pub struct HashMapIter<'a, K, V> {
    iter: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for HashMapIter<'a, K, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }
}

impl<'a> EloStorage<'a, HashMapIter<'a, String, Player>> for HashMap<String, Player> {
    fn add_player(&mut self, player: Player) {
        self.insert(player.name.clone(), player);
    }

    fn update_player(&mut self, player: &Player) {
        self.insert(player.name().to_string(), player.clone());
    }

    fn get(&self, name: &str) -> Option<&Player> {
        self.get(name)
    }

    fn get_mut(&mut self, name: &str) -> Option<&mut Player> {
        self.get_mut(name)
    }

    fn iter(&'a self) -> HashMapIter<'a, String, Player> {
        HashMapIter { iter: self.iter() }
    }
}

#[derive(Debug)]
pub struct Elo<'a, I: Iterator<Item = &'a Player>, S: EloStorage<'a, I>> {
    players: S,
    starting_elo: usize,
    _marker: std::marker::PhantomData<I>,
}

impl<'a, I: Iterator<Item = &'a Player>, S: EloStorage<'a, I>> Elo<'a, I, S> {
    pub fn new(players: S) -> Elo<'a, I, S> {
        Elo {
            players,
            starting_elo: 1000,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add_player<TS: ToString>(&mut self, name: TS) {
        self.players.add_player(Player {
            name: name.to_string(),
            rating: self.starting_elo,
            number_of_games: 0,
        });
    }

    pub fn try_add(&mut self, name: &str) {
        if self.players.get(name).is_none() {
            self.add_player(name);
        }
    }

    /// If is_draw is true, the game is a draw.
    /// If is_draw is false, the game is won by the first player.
    pub fn add_game(&mut self, player1: &str, player2: &str, is_draw: bool) -> Result<(), String> {
        if player1 == player2 {
            return Err(format!(
                "{} can't play against themselves (you friendless loser)",
                player1
            ));
        }

        self.try_add(player1);
        self.try_add(player2);
        let w = &self[player1];
        let l = &self[player2];

        let winner_expected =
            1.0 / (1.0 + 10.0_f64.powf((l.rating as isize - w.rating as isize) as f64 / 400.0));
        let loser_expected =
            1.0 / (1.0 + 10.0_f64.powf((w.rating as isize - l.rating as isize) as f64 / 400.0));

        let factor = if is_draw { 0.5 } else { 0.0 };

        let winner_new_rating = w.rating() as f64 + 32.0 * (1.0 - factor - winner_expected);
        let loser_new_rating = l.rating() as f64 + 32.0 * (factor - loser_expected);

        let mut update_rating = |player, new_rating: f64| {
            let p = self.players.get_mut(player).unwrap();
            p.rating = new_rating.round() as usize;
            p.number_of_games += 1;
        };

        update_rating(player1, winner_new_rating);
        update_rating(player2, loser_new_rating);

        Ok(())
    }

    pub fn get_player(&self, name: &str) -> Option<&Player> {
        self.players.get(name)
    }
}

impl<'a, I: Iterator<Item = &'a Player>, S: EloStorage<'a, I>> Index<&str> for Elo<'a, I, S> {
    type Output = Player;

    fn index(&self, name: &str) -> &Player {
        self.players.get(name).unwrap()
    }
}

impl<'a, I: Iterator<Item = &'a Player>, S: EloStorage<'a, I>> IndexMut<&str> for Elo<'a, I, S> {
    fn index_mut(&mut self, name: &str) -> &mut Player {
        self.players.get_mut(name).unwrap()
    }
}

impl<'a, I: Iterator<Item = &'a Player>, S: EloStorage<'a, I>> IntoIterator for &'a Elo<'a, I, S> {
    type Item = &'a Player;
    type IntoIter = I;

    fn into_iter(self) -> Self::IntoIter {
        self.players.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_no_friends() {
        let mut elo = Elo::new(HashMap::new());
        elo.add_player("a");

        assert!(elo.add_game("a", "a", false).is_err());
    }

    #[test]
    fn dual() {
        let mut elo = Elo::new(HashMap::new());
        elo.add_player("a");
        elo.add_player("b");

        elo.add_game("a", "b", false).unwrap();
        elo.add_game("b", "a", false).unwrap();

        assert_eq!(elo["a"].rating(), 999);
        assert_eq!(elo["b"].rating(), 1001);

        assert_eq!(elo["a"].numer_of_games(), 2);
        assert_eq!(elo["b"].numer_of_games(), 2);
    }

    #[test]
    fn dual_draw() {
        let mut elo = Elo::new(HashMap::new());
        elo.add_player("a");
        elo.add_player("b");

        elo.add_game("a", "b", true).unwrap();

        assert_eq!(elo["a"].rating(), 1000);
        assert_eq!(elo["b"].rating(), 1000);
    }

    #[test]
    fn ordering() {
        let mut elo = Elo::new(HashMap::new());
        elo.add_player("a");
        elo.add_player("b");
        elo.add_player("c");
        elo.add_player("d");

        elo.add_game("a", "b", false).unwrap();
        elo.add_game("a", "b", false).unwrap();
        elo.add_game("a", "c", false).unwrap();

        // force b rating, to see ordering with comparison of c
        elo["b"].rating = 985;

        // add player d, see check that name is ordered lexicographically
        elo["d"].rating = 985;
        elo["d"].number_of_games = 2;

        let mut players = elo.into_iter().collect::<Vec<_>>();
        players.sort();
        assert_eq!(players[0].name(), "a");
        assert_eq!(players[1].name(), "b");
        assert_eq!(players[2].name(), "d");
        assert_eq!(players[3].name(), "c");
    }
}
