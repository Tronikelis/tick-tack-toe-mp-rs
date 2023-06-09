use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Player {
    pub tile: Tile,
    pub addr: Option<String>,
    pub game_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Tile {
    X,
    O,
}

impl Tile {
    pub fn inverse(&self) -> Self {
        return match self {
            Self::O => Self::X,
            Self::X => Self::O,
        };
    }
}

impl ToString for Tile {
    fn to_string(&self) -> String {
        return match self {
            Self::O => "O".to_string(),
            Self::X => "X".to_string(),
        };
    }
}

// player == tile
impl PartialEq<Player> for Tile {
    fn eq(&self, other: &Player) -> bool {
        return self == &other.tile;
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Board {
    pub tiles: [Option<Tile>; 9],
    pub turn: Tile,
}

#[derive(Clone, Debug)]
pub struct GameInstance {
    pub id: String,
    pub board: Board,
    pub players: [Player; 2],
}

impl Board {
    // this gets used in ttt_client
    #[allow(dead_code)]
    pub fn print_board(&self) -> String {
        let mut string = String::new();
        for (i, tile) in self.tiles.iter().enumerate() {
            if i % 3 == 0 {
                string.push('\n');
            }

            match tile {
                None => string.push_str(" - "),
                Some(tile) => string.push_str(&format!(" {} ", tile.to_string())),
            }
        }

        return string;
    }
}

impl GameInstance {
    pub fn new(id: String, players: [Player; 2]) -> Result<Self> {
        let board = Board {
            turn: Tile::X,
            tiles: [(); 9].map(|_| None),
        };

        if players[0].tile == players[1].tile {
            return Err(anyhow!("players have to be separate!"));
        }

        return Ok(Self { id, board, players });
    }

    pub fn set_tile(&mut self, tile_idx: usize, tile: Tile) -> Result<()> {
        match self.board.tiles.get_mut(tile_idx) {
            Some(current_tile) => {
                if current_tile.is_none() {
                    *current_tile = Some(tile);
                    self.board.turn = self.board.turn.inverse();

                    return Ok(());
                }

                return Err(anyhow!("tile already set!"));
            }
            None => return Err(anyhow!("tile_idx out of bounds")),
        };
    }

    /// which player won
    pub fn check_wins(&self) -> Option<Player> {
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

    pub fn add_player(&mut self, addr: String) -> Result<Player> {
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
                    game_id: self.id.clone(),
                    addr: Some(addr),
                    tile: match existing_player
                        .as_ref()
                        .ok_or(anyhow!("can't add a player if there are 0 players"))?
                        .tile
                    {
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

    pub fn get_player(&self, addr: String) -> Option<Player> {
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
