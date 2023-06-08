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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct Player {
    tile: Tile,
    addr: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum Tile {
    X,
    O,
}

// player == tile
impl PartialEq<Player> for Tile {
    fn eq(&self, other: &Player) -> bool {
        return self == &other.tile;
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Board {
    tiles: [Option<Tile>; 9],
}

#[derive(Clone, Debug)]
struct GameInstance {
    id: String,
    board: Board,
    players: [Player; 2],
}

impl GameInstance {
    fn new(id: String, players: [Player; 2]) -> Result<Self> {
        let board = Board {
            tiles: [(); 9].map(|_| None),
        };

        if players[0].tile == players[1].tile {
            return Err(anyhow!("players have to be separate!"));
        }

        return Ok(Self { id, board, players });
    }

    fn print_board(&self) -> String {
        let mut string = String::new();
        for (i, tile) in self.board.tiles.iter().enumerate() {
            if i % 3 == 0 {
                string.push('\n');
            }

            match tile {
                None => string.push_str(" _ "),
                Some(tile) => match tile {
                    Tile::O => string.push_str(" O "),
                    Tile::X => string.push_str(" X "),
                },
            }
        }

        return string;
    }

    fn set_tile(&mut self, tile_idx: usize, tile: Tile) -> Result<()> {
        match self.board.tiles.get_mut(tile_idx) {
            Some(current_tile) => {
                if current_tile.is_none() {
                    *current_tile = Some(tile);
                    return Ok(());
                }

                return Err(anyhow!("tile already set!"));
            }
            None => return Err(anyhow!("tile_idx out of bounds")),
        };
    }

    /// which player won
    fn check_wins(&self) -> Option<Player> {
        // horizontal / vertical
        for player in &self.players {
            for y in 0..3 {
                let mut count_horizontal = 0;
                let mut count_vertical = 0;

                for x in 0..3 {
                    let index_horizontal = y * 3 + x;
                    let index_vertical = x * 3 + y;

                    if let Some(tile) = &self.board.tiles[index_horizontal] {
                        if tile != player {
                            continue;
                        }

                        count_horizontal += 1;
                    }

                    if let Some(tile) = &self.board.tiles[index_vertical] {
                        if tile != player {
                            continue;
                        }

                        count_vertical += 1;
                    }
                }

                if count_horizontal == 3 || count_vertical == 3 {
                    return Some(player.clone());
                }
            }
        }

        // diagonal (\/)
        for player in &self.players {
            for indexes in [[0, 4, 8], [2, 4, 6]] {
                let mut count = 0;
                for i in indexes {
                    if let Some(tile) = &self.board.tiles[i] {
                        if tile == player {
                            count += 1;
                        }
                    }
                }
                if count == 3 {
                    return Some(player.clone());
                }
            }
        }

        return None;
    }

    fn add_player(&mut self, addr: String) -> Result<Player> {
        let mut existing_player = None;

        for player in self.players.clone() {
            if let Some(player_addr) = &player.addr {
                if player_addr == &addr {
                    return Err(anyhow!("todo"));
                }

                existing_player = Some(player);
            }
        }

        for player in &mut self.players {
            if player.addr.is_none() {
                let added = Player {
                    addr: Some(addr),
                    tile: match existing_player.as_ref().unwrap().tile {
                        Tile::O => Tile::X,
                        Tile::X => Tile::O,
                    },
                };

                *player = added.clone();

                return Ok(added);
            }
        }

        return Err(anyhow!("todo"));
    }

    fn get_player(&self, addr: String) -> Option<Player> {
        for player in &self.players {
            if let Some(player_addr) = &player.addr {
                if player_addr == &addr {
                    return Some(player.clone());
                }
            }
        }

        return None;
    }
}

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
    GetGame(String),
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

                ClientCommand::GetGame(id) => {
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
