use crate::Player;

use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

pub trait EloStorage {
    fn add_player(&mut self, player: Player);
    fn update_player(&mut self, player: &Player);
    fn get(&self, name: &str) -> Option<&Player>;
    fn get_mut(&mut self, name: &str) -> Option<&mut Player>;
}

#[derive(Debug)]
pub struct Elo<S: EloStorage> {
    players: S,
    starting_elo: usize,
}

impl<S: EloStorage> Elo<S> {
    #[allow(dead_code)]
    pub fn new(players: S) -> Elo<S> {
        Elo {
            players,
            starting_elo: 1000,
        }
    }

    #[allow(dead_code)]
    pub fn add_player<TS: ToString>(&mut self, name: TS) {
        self.players.add_player(Player {
            name: name.to_string(),
            rating: self.starting_elo,
            number_of_games: 0,
        });
    }

    #[allow(dead_code)]
    pub fn try_add(&mut self, name: &str) {
        if self.players.get(name).is_none() {
            self.add_player(name);
        }
    }

    /// If is_draw is true, the game is a draw.
    /// If is_draw is false, the game is won by the first player.
    #[allow(dead_code)]
    pub fn add_game(&mut self, player1: &str, player2: &str, is_draw: bool) -> Result<(), String> {
        if player1 == player2 {
            return Err(format!(
                "{} can't play against themselves (you friendless loser)",
                player1
            ));
        }

        self.try_add(player1);
        self.try_add(player2);

        let (wr, lr) = crate::update_rating(&self[player1], &self[player2], is_draw);

        let mut update_rating = |player, new_rating: usize| {
            let p = self.players.get_mut(player).unwrap();
            p.rating = new_rating;
            p.number_of_games += 1;
        };

        update_rating(player1, wr);
        update_rating(player2, lr);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_player(&self, name: &str) -> Option<&Player> {
        self.players.get(name)
    }

    #[allow(dead_code)]
    pub fn into_storage(self) -> S {
        self.players
    }
}

impl<S: EloStorage> Index<&str> for Elo<S> {
    type Output = Player;

    fn index(&self, name: &str) -> &Player {
        self.players.get(name).unwrap()
    }
}

impl<S: EloStorage> IndexMut<&str> for Elo<S> {
    fn index_mut(&mut self, name: &str) -> &mut Player {
        self.players.get_mut(name).unwrap()
    }
}

impl EloStorage for HashMap<String, Player> {
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

        assert_eq!(elo["a"].number_of_games(), 2);
        assert_eq!(elo["b"].number_of_games(), 2);
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

        let hm = elo.into_storage();
        let mut players = hm.iter().map(|(_, v)| v).collect::<Vec<_>>();
        players.sort();
        assert_eq!(players[0].name(), "a");
        assert_eq!(players[1].name(), "b");
        assert_eq!(players[2].name(), "d");
        assert_eq!(players[3].name(), "c");
    }
}
