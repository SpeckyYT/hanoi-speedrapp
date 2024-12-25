use std::time::Instant;

use eframe::egui::{self, Context, Response};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{highscores::Score, GameState, HanoiApp};

#[derive(Debug, Default, Clone, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum PlayerKind {
    #[default]
    Human,
    Bot,
    Replay(Score, usize),
}

impl HanoiApp {
    pub fn full_move(&mut self, from: usize, to: usize) {
        if !matches!(self.state, GameState::Finished(_)) {
            if self.hanoi.shift(from, to) {
                if self.state == GameState::Reset {
                    self.state = GameState::Playing(Instant::now());
                }
                self.moves += 1;
                if let GameState::Playing(time) = self.state {
                    self.hanoi.moves_history.push((time.elapsed(), from, to));
                }
            } else if self.reset_on_invalid_move {
                self.soft_reset();
            }
        }
    }

    pub fn player_play(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            for qki in 0..self.quick_keys.len() {
                let (key, from, to) = self.quick_keys[qki];
                if i.key_pressed(key) {
                    self.full_move(from - 1, to - 1);
                    self.reset_undo();
                }
            }

            if matches!((&self.player, &self.state), (PlayerKind::Human, GameState::Playing(_))) && i.key_pressed(self.undo_key) {
                self.undo_move();
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

    pub fn drag_and_drop_play(&mut self, ctx: &Context, poles: Vec<Response>) {
        if matches!(self.state, GameState::Finished(_)) {
            self.dragging_pole = None;
            return;
        }

        match self.dragging_pole {
            None => {
                poles.iter().enumerate().for_each(|(i, pole)| {
                    if pole.drag_started() {
                        self.dragging_pole = Some(i);
                    }
                });
            },
            Some(from) => {
                if poles[from].drag_stopped() {
                    let pointer_position = ctx.input(|i| i.pointer.latest_pos()).unwrap_or_default();
                    poles.iter().enumerate().for_each(|(to, pole)| {
                        if from != to && pole.rect.contains(pointer_position) {
                            self.full_move(from, to);
                            self.reset_undo();
                        }
                    });
                    self.dragging_pole = None;
                }
            },
        }
    }

    pub fn undo_move(&mut self) {
        if let Some((_, from, to)) = self.undo_index.checked_sub(1).and_then(|i| self.hanoi.moves_history.get(i)) {
            self.full_move(*to, *from);
            self.undo_index -= 1;
        }
    }

    #[inline]
    pub fn reset_undo(&mut self) {
        self.undo_index = self.hanoi.moves_history.len();
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

    pub fn replay_play(&mut self) {
        if let PlayerKind::Replay(ref game, ref mut index) = self.player {
            if let Some((time, from, to)) = game.moves.get(*index) {
                if let GameState::Playing(start) = self.state {
                    if start.elapsed() >= *time {
                        self.hanoi.shift(*from, *to);
                        *index += 1;
                        self.moves += 1;
                        if *index >= game.moves.len() {
                            self.state = GameState::Finished(game.time);
                        }
                    }
                }
            }
        }
    }
}
