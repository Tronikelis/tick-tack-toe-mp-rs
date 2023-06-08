use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ClientRequest {
    CreateGame,
    JoinGame(String),
    SetTile((String, usize)),
    GetBoard(String),
}
