use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Group {
    party_owner: usize,
    current_players: Vec<usize>,
    player_amount: usize,
    max_players: usize,
    description: String,
    title: String,
    game: String,
    pub time_til_auto_del: usize
}

impl Group {
    pub(crate) fn new<S: Into<String>>(
        owner: usize,
        max_players: usize,
        title: S,
        game: S
    ) -> Self {
        let mut group = Self::default();
        group.set_owner(owner);
        group.max_players(max_players);
        group.set_game(game);
        group.set_title(title);
        group
    }

    pub(crate) fn add_player(&mut self, player: usize) {
        self.current_players.push(player);
        self.player_amount += 1;
    }

    pub(crate) fn remove_player(&mut self, player: usize) {
        // O(n)
        // NOTE: This vector could also be sorted in the future, in which we can find the player id
        // in O(log(n)) time, but since most of the time the party will not be so many players
        // it's okay to search linearly
        for (i, curr_player) in self.current_players.iter().enumerate() {
            if *curr_player == player {
                self.current_players.remove(i);
                self.player_amount -= 1;
                break
            }
        }
    }

    pub(crate) fn in_player_vec(&self, player: &usize) -> bool {
        for curr_player in self.current_players.iter() { if curr_player == player { return true } }
        false
    }

    pub(crate) fn set_title<T: Into<String>>(&mut self, title: T) {
        self.title = title.into();
    }

    pub(crate) fn set_game<G: Into<String>>(&mut self, game: G) {
        self.title = game.into();
    }

    pub(crate) fn max_players(&mut self, players: usize) {
        self.max_players = players;
    }

    pub(crate) fn set_owner(&mut self, owner: usize) {
        self.party_owner = owner;
    }
}

impl Default for Group {
    fn default() -> Self {
        Self {
            party_owner: 0,
            current_players: Vec::new(),
            player_amount: 0,
            max_players: 0,
            description: String::new(),
            title: String::new(),
            game: String::new(),
            time_til_auto_del: 0
        }
    }
}