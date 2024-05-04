use std::time::{Duration, Instant};

use display::{ColorTheme, PolesPosition};
use eframe::{egui::{self, CentralPanel}, App, Frame, HardwareAcceleration, NativeOptions};
use highscores::Highscores;
use play::PlayerKind;
use serde::{Deserialize, Serialize};
use hanoi::HanoiGame;

mod hanoi;
mod display;
mod play;
mod highscores;
mod util;

const APP_NAME: &str = "Towers of Hanoi - Speedrapp Edition";

fn main() -> Result<(), eframe::Error> {
    HanoiApp::run()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum GameState {
    #[serde(skip)]
    Playing(Instant),
    Finished(Duration),
    Reset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HanoiApp {
    hanoi: HanoiGame,
    player: PlayerKind,
    state: GameState,
    moves: u128,

    // display
    blindfold: bool,
    show_poles: bool,
    disk_number: bool,
    color_theme: ColorTheme,
    poles_position: PolesPosition,

    // other
    extra_mode: bool,

    highscores: Highscores,
}

impl Default for HanoiApp {
    fn default() -> Self {
        Self {
            hanoi: Default::default(),
            player: PlayerKind::Human,
            state: GameState::Reset,
            moves: 0,

            blindfold: false,
            show_poles: true,
            disk_number: false,
            color_theme: ColorTheme::Rainbow,
            poles_position: PolesPosition::Bottom,

            extra_mode: false,

            highscores: Default::default(),
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
}

impl App for HanoiApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.check_extra_mode(ctx);

        match self.player {
            PlayerKind::Human => self.player_play(ctx),
            PlayerKind::Bot => self.bot_play(),
            PlayerKind::Replay => todo!(),
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
            ui.separator();
        });

        if matches!(self.state, GameState::Playing(_)) {
            ctx.request_repaint();
        }
    }
}
