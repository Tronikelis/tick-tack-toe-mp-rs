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
    server::{send_board, send_nothing, send_player},
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
                serde_json::from_str(&read_from_stream(&mut stream).unwrap()).unwrap();

            match client_command {
                // creates a game
                // returns a player
                ClientRequest::CreateGame => {
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
                    send_player(&mut stream, client_player).unwrap();
                }

                // joins a game
                // returns a player
                ClientRequest::JoinGame(id) => {
                    let mut games_locked = games.lock().unwrap();
                    let game_instance = games_locked.get_mut(&id).unwrap();

                    let player = game_instance.add_player(addr.to_string()).unwrap();
                    send_player(&mut stream, player).unwrap();
                }

                // sets tile
                // returns nothing
                ClientRequest::SetTile((id, tile_idx)) => {
                    let mut games_locked = games.lock().unwrap();
                    let game_instance = games_locked.get_mut(&id).unwrap();

                    let player = game_instance.get_player(addr.to_string()).unwrap();

                    game_instance.set_tile(tile_idx, player.tile).unwrap();
                    send_nothing(&mut stream).unwrap();
                }

                // returns the board in just plain text
                ClientRequest::GetBoard(id) => {
                    let games_locked = games.lock().unwrap();
                    let game_instance = games_locked.get(&id).unwrap();

                    send_board(&mut stream, game_instance.print_board()).unwrap();
                }
            }
        });
    }
}
