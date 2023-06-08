#![allow(clippy::needless_return)]

use anyhow::{anyhow, Result};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

mod game;
use game::instance::{Board, GameInstance, Player, Tile};

fn read_from_stream(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 64];

    if stream.read(&mut buffer).unwrap() < buffer.len() {
        return String::from_utf8(buffer.to_vec()).unwrap();
    }

    let mut vec_buffer = vec![];
    while stream.read(&mut buffer).unwrap() > 0 {
        for x in buffer.iter().filter(|x| **x != 0) {
            vec_buffer.push(*x);
        }
    }

    return String::from_utf8(vec_buffer).unwrap();
}

#[derive(Serialize, Deserialize)]
enum ClientCommand {
    CreateGame,
    JoinGame(String),
    SetTile((String, usize)),
    GetBoard(String),
}

fn main() {
    let games: Arc<Mutex<HashMap<String, GameInstance>>> = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();

    loop {
        let (mut stream, addr) = listener.accept().unwrap();
        let games = games.clone();

        thread::spawn(move || loop {
            let client_command: ClientCommand =
                serde_json::from_str(&read_from_stream(&mut stream)).unwrap();

            match client_command {
                // creates a game
                // returns a player
                ClientCommand::CreateGame => {
                    let id = nanoid!();

                    let client_player = Player {
                        addr: Some(addr.to_string()),
                        tile: Tile::X,
                    };

                    let game_instance = GameInstance::new(
                        id.clone(),
                        [
                            client_player.clone(),
                            Player {
                                addr: None,
                                tile: Tile::O,
                            },
                        ],
                    )
                    .unwrap();

                    games.lock().unwrap().insert(id.clone(), game_instance);

                    stream
                        .write_all(serde_json::to_string(&client_player).unwrap().as_bytes())
                        .unwrap();
                }

                // joins a game
                // returns a player
                ClientCommand::JoinGame(id) => {
                    let mut games_locked = games.lock().unwrap();
                    let game_instance = games_locked.get_mut(&id).unwrap();

                    let player = game_instance.add_player(addr.to_string()).unwrap();

                    stream
                        .write_all(serde_json::to_string(&player).unwrap().as_bytes())
                        .unwrap();
                }

                // sets tile
                // returns nothing
                ClientCommand::SetTile((id, tile_idx)) => {
                    let mut games_locked = games.lock().unwrap();
                    let game_instance = games_locked.get_mut(&id).unwrap();

                    let player = game_instance.get_player(addr.to_string()).unwrap();

                    game_instance.set_tile(tile_idx, player.tile).unwrap();
                }

                // returns the board in just plain text
                ClientCommand::GetBoard(id) => {
                    let games_locked = games.lock().unwrap();
                    let game_instance = games_locked.get(&id).unwrap();

                    stream
                        .write_all(game_instance.print_board().as_bytes())
                        .unwrap();
                }
            }
        });
    }
}
