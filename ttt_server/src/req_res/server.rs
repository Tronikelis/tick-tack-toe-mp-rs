use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{io::Write, net::TcpStream};

use crate::game::instance::{Board, Player};

#[derive(Serialize, Deserialize)]
pub enum ServerResponse {
    Error(String),
    Nothing,

    Player(Player),
    GameLoop(GameLoop),
}

#[derive(Serialize, Deserialize)]
pub enum GameLoop {
    Board(Board),
    Won(Player),
}

pub fn send_player(stream: &mut TcpStream, player: Player) -> Result<()> {
    stream.write_all(serde_json::to_string(&ServerResponse::Player(player))?.as_bytes())?;
    return Ok(());
}

pub fn send_nothing(stream: &mut TcpStream) -> Result<()> {
    stream.write_all(serde_json::to_string(&ServerResponse::Nothing)?.as_bytes())?;
    return Ok(());
}

pub fn send_error(stream: &mut TcpStream, error: String) -> Result<()> {
    stream.write_all(serde_json::to_string(&ServerResponse::Error(error))?.as_bytes())?;
    return Ok(());
}

pub fn send_game_loop(stream: &mut TcpStream, game_loop: GameLoop) -> Result<()> {
    stream.write_all(serde_json::to_string(&ServerResponse::GameLoop(game_loop))?.as_bytes())?;
    return Ok(());
}
