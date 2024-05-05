use std::time::Instant;

use eframe::egui::{self, Key};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{GameState, HanoiApp};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum PlayerKind {
    #[default]
    Human,
    Bot,
    Replay,
}

impl HanoiApp {
    pub fn player_play(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            macro_rules! inputs {
                ($($k:ident: $f:literal => $t:literal)*) => {
                    $(
                        if i.key_pressed(Key::$k) {
                            if !matches!(self.state, GameState::Finished(_)) {
                                if self.hanoi.shift($f - 1, $t - 1) {
                                    if self.state == GameState::Reset {
                                        self.state = GameState::Playing(Instant::now());
                                    }
                                    self.moves += 1;
                                    if let GameState::Playing(time) = self.state {
                                        self.hanoi.moves_history.push((time.elapsed(), $f - 1, $t - 1));
                                    }
                                }
                            }
                        }
                    )*
                };
            }

            inputs!(
                D: 1 => 2
                F: 1 => 3
                S: 2 => 1
                L: 2 => 3
                J: 3 => 1
                K: 3 => 2
            );

            if i.key_pressed(Key::R) {
                self.soft_reset();
            }
        });

        match self.state {
            GameState::Playing(start) if self.hanoi.finished() => {
                let elapsed = start.elapsed();
                self.state = GameState::Finished(elapsed);
                self.save_score(elapsed);
            },
            _ => {},
        }
    }

    pub fn bot_play(&mut self) {
        if self.state == GameState::Reset {
            let start_time = Instant::now();
            self.state = GameState::Playing(start_time);
            self.moves = 0;
            fn hanoi_bot(game: &mut HanoiApp, n: usize, from_rod: usize, to_rod: usize, aux_rod: usize) {
                if n > 0 {
                    hanoi_bot(game, n - 1, from_rod, aux_rod, to_rod);
                    if game.hanoi.shift(from_rod, to_rod) {
                        game.moves += 1;
                    }
                    hanoi_bot(game, n - 1, aux_rod, to_rod, from_rod);
                }
            }
            hanoi_bot(
                self,
                self.hanoi.disks_count,
                self.hanoi.start_pole - 1,
                (self.hanoi.end_pole.unwrap_or(self.hanoi.start_pole)) % self.hanoi.poles_count,
                (self.hanoi.end_pole.unwrap_or(self.hanoi.start_pole + 1)) % self.hanoi.poles_count,
            );
            self.state = GameState::Finished(start_time.elapsed());
        }
    }
}
