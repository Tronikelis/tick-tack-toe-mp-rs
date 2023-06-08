#![allow(clippy::needless_return)]

use anyhow::Result;
use std::{
    io::{self, BufRead, Write},
    net::TcpStream,
    thread,
    time::Duration,
};
use ttt_server::{
    game::instance::Player,
    req_res::{
        client::ClientRequest,
        server::{GameLoop, ServerResponse},
    },
    utils::stream::read_from_stream,
};

fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    io::stdin().lock().read_line(&mut buffer)?;
    return Ok(buffer.trim().to_string());
}

fn clear_stdin() {
    print!("\x1B[2J");
}

fn game_loop(stream: &mut TcpStream, player: Player) -> Result<()> {
    let game_id = &player.game_id;
    let mut prev_board_str = String::new();

    loop {
        thread::sleep(Duration::from_millis(16));

        stream.write_all(
            serde_json::to_string(&ClientRequest::GameLoop(game_id.clone()))?.as_bytes(),
        )?;

        let response: ServerResponse = serde_json::from_str(&read_from_stream(stream)?)?;

        if let ServerResponse::GameLoop(game_loop) = response {
            match game_loop {
                GameLoop::Board(board) => {
                    let board_str = board.print_board();
                    if prev_board_str == board_str {
                        continue;
                    }
                    prev_board_str = board_str.clone();

                    clear_stdin();
                    println!("game_id: {}", game_id);
                    println!("\n{}\n", board_str);
                    println!("You are {}", player.tile.to_string());

                    if board.turn == player {
                        println!("Your choice?");
                        let selection: usize = read_stdin()?.parse()?;

                        stream.write_all(
                            serde_json::to_string(&ClientRequest::SetTile((
                                game_id.clone(),
                                selection,
                            )))?
                            .as_bytes(),
                        )?;
                        read_from_stream(stream)?;
                        continue;
                    }

                    println!("Waiting for opponent");
                }
                GameLoop::Won(tile) => {
                    if tile == player {
                        println!("You won");
                    } else {
                        println!("You lost");
                    }

                    return Ok(());
                }
            };
        }
    }
}

fn main() -> Result<()> {
    let server_ip = "127.0.0.1:3000";

    loop {
        let mut stream = TcpStream::connect(server_ip).unwrap();

        println!("game [create / join]?");

        match read_stdin().unwrap().as_str() {
            "create" => {
                stream.write_all(serde_json::to_string(&ClientRequest::CreateGame)?.as_bytes())?;
                let response: ServerResponse =
                    serde_json::from_str(&read_from_stream(&mut stream)?)?;

                if let ServerResponse::Player(player) = response {
                    if let Err(err) = game_loop(&mut stream, player) {
                        println!("RESTARTING: {}", err);
                        continue;
                    }
                } else {
                    panic!("incorrect server response, expected player to be returned");
                }
            }
            "join" => {
                println!("id [*game_id*]?");
                let game_id = read_stdin()?;

                stream.write_all(
                    serde_json::to_string(&ClientRequest::JoinGame(game_id))?.as_bytes(),
                )?;

                let response: ServerResponse =
                    serde_json::from_str(&read_from_stream(&mut stream)?)?;

                if let ServerResponse::Player(player) = response {
                    if let Err(err) = game_loop(&mut stream, player) {
                        println!("RESTARTING: {}", err);
                        continue;
                    }
                } else {
                    panic!("incorrect server response, expected player to be returned");
                }
            }
            _ => {
                panic!("unknown command")
            }
        }
    }
}
