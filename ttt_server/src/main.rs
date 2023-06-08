#![allow(clippy::needless_return)]

use nanoid::nanoid;
use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

mod game;
use game::instance::{GameInstance, Player, Tile};

mod req_res;
use req_res::{
    client::ClientRequest,
    server::{send_error, send_game_loop, send_nothing, send_player, GameLoop},
};

mod utils;
use utils::stream::read_from_stream;

fn main() {
    let games: Arc<Mutex<HashMap<String, GameInstance>>> = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();

    loop {
        let (mut stream, addr) = listener.accept().unwrap();
        let games = games.clone();

        thread::spawn(move || loop {
            let client_command: ClientRequest =
                match serde_json::from_str(&read_from_stream(&mut stream).unwrap()) {
                    Ok(x) => x,
                    Err(_) => return,
                };

            match client_command {
                // creates a game
                // returns a (player, id)
                ClientRequest::CreateGame => {
                    let id = nanoid!();

                    let client_player = Player {
                        game_id: id.clone(),
                        addr: Some(addr.to_string()),
                        tile: Tile::X,
                    };

                    let game_instance = match GameInstance::new(
                        id.clone(),
                        [
                            client_player.clone(),
                            Player {
                                addr: None,
                                tile: Tile::O,
                                game_id: id.clone(),
                            },
                        ],
                    ) {
                        Ok(x) => x,
                        Err(err) => {
                            send_error(&mut stream, err.to_string()).unwrap();
                            return;
                        }
                    };

                    games.lock().unwrap().insert(id.clone(), game_instance);
                    send_player(&mut stream, client_player).unwrap();
                }

                // joins a game
                // returns a player
                ClientRequest::JoinGame(id) => {
                    let mut games_locked = games.lock().unwrap();
                    let game_instance = match games_locked.get_mut(&id) {
                        Some(x) => x,
                        None => {
                            send_error(&mut stream, "game not found".to_string()).unwrap();
                            return;
                        }
                    };

                    let player = match game_instance.add_player(addr.to_string()) {
                        Ok(x) => x,
                        Err(err) => {
                            send_error(&mut stream, err.to_string()).unwrap();
                            return;
                        }
                    };

                    send_player(&mut stream, player).unwrap();
                }

                // sets tile
                // returns nothing
                ClientRequest::SetTile((id, tile_idx)) => {
                    let mut games_locked = games.lock().unwrap();
                    let game_instance = match games_locked.get_mut(&id) {
                        Some(x) => x,
                        None => {
                            send_error(&mut stream, "game not found".to_string()).unwrap();
                            return;
                        }
                    };

                    let player = match game_instance.get_player(addr.to_string()) {
                        Some(x) => x,
                        None => {
                            send_error(&mut stream, "player not found".to_string()).unwrap();
                            return;
                        }
                    };

                    match game_instance.set_tile(tile_idx, player.tile) {
                        Ok(_) => {}
                        Err(err) => {
                            send_error(&mut stream, err.to_string()).unwrap();
                            return;
                        }
                    };

                    send_nothing(&mut stream).unwrap();
                }

                // returns the board in just plain text
                ClientRequest::GameLoop(id) => {
                    let mut games_locked = games.lock().unwrap();
                    let game_instance = match games_locked.get_mut(&id) {
                        Some(x) => x,
                        None => {
                            send_error(&mut stream, "game not found".to_string()).unwrap();
                            return;
                        }
                    };

                    if let Some(player) = game_instance.check_wins() {
                        send_game_loop(&mut stream, GameLoop::Won(player)).unwrap();
                        return;
                    }

                    send_game_loop(&mut stream, GameLoop::Board(game_instance.print_board()))
                        .unwrap();
                }
            }
        });
    }
}
