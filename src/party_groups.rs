use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Group {
    pub party_owner: i64,
    current_players: Vec<i64>,
    player_names: Vec<String>,
    player_amount: i64,
    max_players: i64,
    title: String,
    game: String,
    voice_id: i64,
    text_id: i64,
    role_id: i64,
    pub time_til_auto_del: i64
}

impl Group {
    pub(crate) async fn new<S: Into<String>>(
        owner: i64,
        max_players: i64,
        title: S,
        game: S,
        voice: i64,
        text: i64,
        role: i64
    ) -> Self {
        let mut group = Self::default();
        group.set_owner(owner);
        group.max_players(max_players);
        group.set_game(game);
        group.set_title(title);
        group.set_voice_id(voice);
        group.set_text_id(text);
        group.set_role_id(role);
        group
    }

    pub(crate) fn full(&self) -> bool {
        if self.player_amount == self.max_players { true } else { false }
    }

    pub(crate) async fn add_player(&mut self, player: i64) {
        self.current_players.push(player);
        self.player_amount += 1;
    }

    pub(crate) async fn add_player_name(&mut self, player_name: String) {
        self.player_names.push(player_name);
    }

    pub(crate) async fn remove_player(&mut self, player: i64) {
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

    pub(crate) async fn remove_player_name(&mut self, player_name: String) {
        for (i, curr_player) in self.player_names.iter().enumerate() {
            if *curr_player == player_name {
                self.player_names.remove(i);
                break
            }
        }
    }

    pub(crate) fn in_player_vec(&self, player: &i64) -> bool {
        for curr_player in self.current_players.iter() { if curr_player == player { return true } }
        false
    }

    pub(crate) fn set_title<T: Into<String>>(&mut self, title: T) {
        self.title = title.into();
    }

    pub(crate) fn set_game<G: Into<String>>(&mut self, game: G) {
        self.game = game.into();
    }

    pub(crate) fn max_players(&mut self, players: i64) {
        self.max_players = players;
    }

    pub(crate) fn set_owner(&mut self, owner: i64) {
        self.party_owner = owner;
    }

    pub(crate) fn set_voice_id(&mut self, id: i64) {
        self.voice_id = id;
    }

    pub(crate) fn set_text_id(&mut self, id: i64) {
        self.text_id = id;
    }

    pub(crate) fn set_role_id(&mut self, id: i64) {
        self.role_id = id;
    }

    pub(crate) fn players(&self) -> String {
        if self.player_names.len() == 0 { "None".to_string() }
        else {
            let current_string: String = self.player_names
            .clone()
            .into_iter()
            .map(|mut x| { x.push_str(", "); x })
            .collect();

        current_string
            .strip_suffix(", ")
            .unwrap()
            .to_string()
        }
    }
}

impl Default for Group {
    fn default() -> Self {
        Self {
            party_owner: 0,
            current_players: Vec::new(),
            player_names: Vec::new(),
            player_amount: 0,
            max_players: 0,
            title: String::new(),
            game: String::new(),
            voice_id: 0,
            text_id: 0,
            role_id: 0,
            time_til_auto_del: 0
        }
    }
}