use async_trait::async_trait;

use crate::Player;

use std::collections::HashMap;

#[async_trait]
pub trait AsyncEloStorage {
    async fn add_player(&mut self, player: Player);
    async fn update_player(&mut self, player: &Player);
    async fn get(&self, name: &str) -> Option<&Player>;
    async fn get_mut(&mut self, name: &str) -> Option<&mut Player>;
}

#[derive(Debug)]
pub struct AsyncElo<S: AsyncEloStorage> {
    players: S,
    starting_elo: usize,
}

impl<S: AsyncEloStorage> AsyncElo<S> {
    #[allow(dead_code)]
    pub fn new(players: S) -> AsyncElo<S> {
        AsyncElo {
            players,
            starting_elo: 1000,
        }
    }

    #[allow(dead_code)]
    pub async fn add_player<TS: ToString>(&mut self, name: TS) {
        self.players
            .add_player(Player {
                name: name.to_string(),
                rating: self.starting_elo,
                number_of_games: 0,
            })
            .await;
    }

    #[allow(dead_code)]
    pub async fn try_add(&mut self, name: &str) {
        if self.players.get(name).await.is_none() {
            self.add_player(name).await;
        }
    }

    /// If is_draw is true, the game is a draw.
    /// If is_draw is false, the game is won by the first player.
    #[allow(dead_code)]
    pub async fn add_game(
        &mut self,
        player1: &str,
        player2: &str,
        is_draw: bool,
    ) -> Result<(), String> {
        if player1 == player2 {
            return Err(format!(
                "{} can't play against themselves (you friendless loser)",
                player1
            ));
        }

        self.try_add(player1).await;
        self.try_add(player2).await;

        let (wr, lr) = crate::update_rating(
            self.get_player(player1).await.unwrap(),
            self.get_player(player2).await.unwrap(),
            is_draw,
        );

        let mut p1 = self.get_player_mut(player1).await.unwrap();
        p1.rating = wr;
        p1.number_of_games += 1;

        let mut p2 = self.get_player_mut(player2).await.unwrap();
        p2.rating = lr;
        p2.number_of_games += 1;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_player(&self, name: &str) -> Option<&Player> {
        self.players.get(name).await
    }

    #[allow(dead_code)]
    pub async fn get_player_mut(&mut self, name: &str) -> Option<&mut Player> {
        self.players.get_mut(name).await
    }

    #[allow(dead_code)]
    pub fn into_storage(self) -> S {
        self.players
    }
}

#[async_trait]
impl AsyncEloStorage for HashMap<String, Player> {
    async fn add_player(&mut self, player: Player) {
        self.insert(player.name.clone(), player);
    }

    async fn update_player(&mut self, player: &Player) {
        self.insert(player.name().to_string(), player.clone());
    }

    async fn get(&self, name: &str) -> Option<&Player> {
        self.get(name)
    }

    async fn get_mut(&mut self, name: &str) -> Option<&mut Player> {
        self.get_mut(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn single_no_friends() {
        let mut elo = AsyncElo::new(HashMap::new());
        elo.add_player("a").await;

        assert!(elo.add_game("a", "a", false).await.is_err());
    }

    #[tokio::test]
    async fn dual() {
        let mut elo = AsyncElo::new(HashMap::new());
        elo.add_player("a").await;
        elo.add_player("b").await;

        elo.add_game("a", "b", false).await.unwrap();
        elo.add_game("b", "a", false).await.unwrap();

        assert_eq!(elo.get_player("a").await.unwrap().rating(), 999);
        assert_eq!(elo.get_player("b").await.unwrap().rating(), 1001);

        assert_eq!(elo.get_player("a").await.unwrap().number_of_games(), 2);
        assert_eq!(elo.get_player("b").await.unwrap().number_of_games(), 2);
    }

    #[tokio::test]
    async fn dual_draw() {
        let mut elo = AsyncElo::new(HashMap::new());
        elo.add_player("a").await;
        elo.add_player("b").await;

        elo.add_game("a", "b", true).await.unwrap();

        assert_eq!(elo.get_player("a").await.unwrap().rating(), 1000);
        assert_eq!(elo.get_player("b").await.unwrap().rating(), 1000);
    }

    #[tokio::test]
    async fn ordering() {
        let mut elo = AsyncElo::new(HashMap::new());
        elo.add_player("a").await;
        elo.add_player("b").await;
        elo.add_player("c").await;
        elo.add_player("d").await;

        elo.add_game("a", "b", false).await.unwrap();
        elo.add_game("a", "b", false).await.unwrap();
        elo.add_game("a", "c", false).await.unwrap();

        // force b rating, to see ordering with comparison of c
        elo.get_player_mut("b").await.unwrap().rating = 985;

        // add player d, see check that name is ordered lexicographically
        elo.get_player_mut("d").await.unwrap().rating = 985;
        elo.get_player_mut("d").await.unwrap().number_of_games = 2;

        let hm = elo.into_storage();
        let mut players = hm.iter().map(|(_, v)| v).collect::<Vec<_>>();
        players.sort();
        assert_eq!(players[0].name(), "a");
        assert_eq!(players[1].name(), "b");
        assert_eq!(players[2].name(), "d");
        assert_eq!(players[3].name(), "c");
    }
}
