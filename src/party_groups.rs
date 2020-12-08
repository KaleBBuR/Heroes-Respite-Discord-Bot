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
    time_til_auto_del: usize
}
