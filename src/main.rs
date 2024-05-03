use std::time::Instant;

use eframe::{egui::{self, CentralPanel, Key, Modifiers}, App, Frame, HardwareAcceleration, NativeOptions};
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use hanoi::HanoiGame;

mod hanoi;
mod display;

const APP_NAME: &str = "Towers of Hanoi - Speedrapp Edition";

fn main() -> Result<(), eframe::Error> {
    HanoiApp::run()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum GameState {
    Playing,
    #[serde(skip)]
    Finished(Instant),
    Reset,
}

fn default_instant() -> Instant {
    Instant::now()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HanoiApp {
    hanoi: HanoiGame,
    player: PlayerKind,
    state: GameState,
    #[serde(skip, default = "default_instant")]
    start: Instant,
    moves: u128,

    // display
    blindfold: bool,
    show_poles: bool,
    disk_number: bool,
    color_theme: ColorTheme,
    poles_position: PolesPosition,

    // other
    extra_mode: bool,
}

impl Default for HanoiApp {
    fn default() -> Self {
        Self {
            hanoi: Default::default(),
            player: PlayerKind::Human,
            state: GameState::Reset,
            start: Instant::now(),
            moves: 0,

            blindfold: false,
            show_poles: true,
            disk_number: false,
            color_theme: ColorTheme::Rainbow,
            poles_position: PolesPosition::Bottom,

            extra_mode: false,
        }
    }
}

impl HanoiApp {
    pub fn run() -> Result<(), eframe::Error> {
        let options = NativeOptions {
            hardware_acceleration: HardwareAcceleration::Preferred,
            vsync: false,
            persist_window: true,

            ..Default::default()
        };

        eframe::run_native(
            APP_NAME,
            options,
            Box::new(Self::load),
        )
    }
    fn load(cc: &eframe::CreationContext) -> Box<dyn eframe::App> {
        Box::new(if let Some(storage) = cc.storage {
            let mut app = eframe::get_value::<HanoiApp>(storage, eframe::APP_KEY).unwrap_or_default();
            app.soft_reset();
            app
        } else {
            HanoiApp::default()
        })
    }

    pub fn soft_reset(&mut self) {
        self.hanoi.reset();
        self.state = GameState::Reset;
        self.moves = 0;
    }

    pub fn equal_settings(&self, other: &Self) -> bool {
        self.hanoi.disks_count == other.hanoi.disks_count
            && self.hanoi.end_pole == other.hanoi.end_pole
            && self.hanoi.illegal_moves == other.hanoi.illegal_moves
            && self.hanoi.poles_count == other.hanoi.poles_count
            && self.hanoi.start_pole == other.hanoi.start_pole
            && self.blindfold == other.blindfold
            && self.show_poles == other.show_poles
            && self.disk_number == other.disk_number
            && self.player == other.player
    }
    pub fn check_extra_mode(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            let modifiers = i.modifiers.contains(Modifiers::SHIFT|Modifiers::COMMAND|Modifiers::ALT);
            let space = i.key_down(Key::Enter);
            let mouse = i.pointer.primary_down() && i.pointer.secondary_down();

            if modifiers && space && mouse {
                self.extra_mode = true;
            }
        })
    }
    pub fn player_play(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            macro_rules! inputs {
                ($($k:ident: $f:literal => $t:literal)*) => {
                    $(
                        if i.key_pressed(Key::$k) {
                            if self.state == GameState::Reset {
                                self.state = GameState::Playing;
                                self.start = Instant::now();
                                self.moves = 0;
                            }
                            if self.state == GameState::Playing {
                                if self.hanoi.shift($f - 1, $t - 1) {
                                    self.moves += 1;
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

        if matches!(self.state, GameState::Playing) && self.hanoi.finished() {
            self.state = GameState::Finished(Instant::now());
        }
    }

    pub fn bot_play(&mut self) {
        if self.state == GameState::Reset {
            self.state = GameState::Playing;
            self.start = Instant::now();
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
            self.state = GameState::Finished(Instant::now());
        }
    }
}

impl App for HanoiApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.check_extra_mode(ctx);

        if self.state == GameState::Reset {
            self.hanoi.reset();
        }

        match self.player {
            PlayerKind::Human => self.player_play(ctx),
            PlayerKind::Bot => self.bot_play(),
        };

        if self.blindfold {
            self.draw_blindfold(ctx);
        } else {
            self.draw_poles(ctx);
        }

        CentralPanel::default()
        .show(ctx, |ui| {
            self.draw_settings(ui);
            ui.separator();
            self.draw_state(ui);
            if let GameState::Finished(end) = self.state {
                ui.separator();
                self.draw_completed(ui, end);
            }
        });

        if matches!(self.state, GameState::Playing) {
            ctx.request_repaint();
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
enum PlayerKind {
    Human,
    Bot,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
enum ColorTheme {
    Rainbow,
    Purple,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
enum PolesPosition {
    Bottom,
    Top,
}
