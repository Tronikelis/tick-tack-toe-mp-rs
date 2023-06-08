use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{io::Write, net::TcpStream};

use crate::game::instance::Player;

#[derive(Serialize, Deserialize)]
pub enum ServerResponse {
    Error(String),
    NothingRes,

    PlayerRes(Player),
    BoardRes(String),
}

pub fn send_player(stream: &mut TcpStream, player: Player) -> Result<()> {
    stream.write_all(serde_json::to_string(&ServerResponse::PlayerRes(player))?.as_bytes())?;
    return Ok(());
}

pub fn send_nothing(stream: &mut TcpStream) -> Result<()> {
    stream.write_all(serde_json::to_string(&ServerResponse::NothingRes)?.as_bytes())?;
    return Ok(());
}

pub fn send_board(stream: &mut TcpStream, board: String) -> Result<()> {
    stream.write_all(board.as_bytes())?;
    return Ok(());
}
