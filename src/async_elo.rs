use async_trait::async_trait;

use crate::Player;

use std::collections::HashMap;

use std::sync::RwLock;

#[async_trait]
pub trait AsyncEloStorage {
    /// Add a new player
    async fn add_player(&self, player: Player);
    /// Update the rating and number of games played for a player
    async fn update_player(&self, player: &Player);
    /// Get the player
    async fn get(&self, name: &str) -> Option<Player>;
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
    pub async fn add_player<TS: ToString>(&self, name: TS) {
        self.players
            .add_player(Player {
                name: name.to_string(),
                rating: self.starting_elo,
                number_of_games: 0,
            })
            .await;
    }

    #[allow(dead_code)]
    pub async fn try_add(&self, name: &str) {
        if self.players.get(name).await.is_none() {
            self.add_player(name).await;
        }
    }

    /// If is_draw is true, the game is a draw.
    /// If is_draw is false, the game is won by the first player.
    #[allow(dead_code)]
    pub async fn add_game(
        &self,
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

        let mut player1 = self.get_player(player1).await.unwrap();
        let mut player2 = self.get_player(player2).await.unwrap();

        let (wr, lr) = crate::update_rating(&player1, &player2, is_draw);

        player1.rating = wr;
        player1.number_of_games += 1;

        player2.rating = lr;
        player2.number_of_games += 1;

        self.players.update_player(&player1).await;
        self.players.update_player(&player2).await;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_player(&self, name: &str) -> Option<Player> {
        self.players.get(name).await
    }

    #[allow(dead_code)]
    pub fn into_storage(self) -> S {
        self.players
    }

    #[allow(dead_code)]
    pub async fn set_rating(&self, player: &str, rating: usize) {
        let mut player = self.players.get(player).await.unwrap();
        player.rating = rating;
        self.players.update_player(&player).await;
    }

    #[allow(dead_code)]
    pub async fn set_number_of_games(&self, player: &str, number_of_games: usize) {
        let mut player = self.players.get(player).await.unwrap();
        player.number_of_games = number_of_games;
        self.players.update_player(&player).await;

    }
}

struct InMemoryStorage {
    players: RwLock<HashMap<String, Player>>,
}

impl InMemoryStorage {
    #[allow(dead_code)]
    fn new() -> Self {
        InMemoryStorage {
            players: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl AsyncEloStorage for InMemoryStorage {
    async fn add_player(&self, player: Player) {
        self.players
            .write()
            .unwrap()
            .insert(player.name.clone(), player);
    }

    async fn update_player(&self, player: &Player) {
        self.players
            .write()
            .unwrap()
            .insert(player.name().to_string(), player.clone());
    }

    async fn get(&self, name: &str) -> Option<Player> {
        self.players.read().unwrap().get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn single_no_friends() {
        let elo = AsyncElo::new(InMemoryStorage::new());
        elo.add_player("a").await;

        assert!(elo.add_game("a", "a", false).await.is_err());
    }

    #[tokio::test]
    async fn dual() {
        let elo = AsyncElo::new(InMemoryStorage::new());
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
        let elo = AsyncElo::new(InMemoryStorage::new());
        elo.add_player("a").await;
        elo.add_player("b").await;

        elo.add_game("a", "b", true).await.unwrap();

        assert_eq!(elo.get_player("a").await.unwrap().rating(), 1000);
        assert_eq!(elo.get_player("b").await.unwrap().rating(), 1000);
    }

    #[tokio::test]
    async fn ordering() {
        let elo = AsyncElo::new(InMemoryStorage::new());
        elo.add_player("a").await;
        elo.add_player("b").await;
        elo.add_player("c").await;
        elo.add_player("d").await;

        elo.add_game("a", "b", false).await.unwrap();
        elo.add_game("a", "b", false).await.unwrap();
        elo.add_game("a", "c", false).await.unwrap();

        // force b rating, to see ordering with comparison of c
        elo.set_rating("b", 985).await;

        // add player d, see check that name is ordered lexicographically
        elo.set_rating("d", 985).await;
        elo.set_number_of_games("d", 2).await;

        let hm = elo.into_storage();
        let players = hm.players.read().unwrap();
        let mut players = players.iter().map(|(_, v)| v).collect::<Vec<_>>();
        players.sort();
        assert_eq!(players[0].name(), "a");
        assert_eq!(players[1].name(), "b");
        assert_eq!(players[2].name(), "d");
        assert_eq!(players[3].name(), "c");
    }
}
