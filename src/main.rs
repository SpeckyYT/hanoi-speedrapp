use std::time::{Duration, Instant};

use display::{ColorTheme, PolesPosition};
use eframe::{egui, App, Frame, HardwareAcceleration, NativeOptions};
use highscores::Highscores;
use play::PlayerKind;
use serde::{Deserialize, Serialize};
use hanoi::HanoiGame;
use util::{falsy, truthy};

mod hanoi;
mod display;
mod play;
mod highscores;
mod util;

const PROFILING: bool = false;
const APP_NAME: &str = "Towers of Hanoi - Speedrapp Edition";

fn main() -> Result<(), eframe::Error> {
    enable_profiling();
    HanoiApp::run()
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum GameState {
    #[serde(skip)]
    Playing(Instant),
    Finished(Duration),
    #[default]
    Reset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HanoiApp {
    #[serde(default)]
    hanoi: HanoiGame,
    #[serde(default)]
    player: PlayerKind,
    #[serde(default)]
    state: GameState,
    #[serde(default)]
    moves: u128,

    // display
    #[serde(default = "falsy")]
    blindfold: bool,
    #[serde(default = "truthy")]
    show_poles: bool,
    #[serde(default = "falsy")]
    disk_number: bool,
    #[serde(default)]
    color_theme: ColorTheme,
    #[serde(default)]
    poles_position: PolesPosition,

    // windows
    #[serde(default = "falsy")]
    settings_window: bool,

    // other
    #[serde(skip, default = "falsy")]
    extra_mode: bool,

    #[serde(default)]
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

            settings_window: false,

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
        
        self.draw_top_bar(ctx);
        self.draw_central_panel(ctx);

        if matches!(self.state, GameState::Playing(_)) {
            ctx.request_repaint();
        }
    }
}

fn enable_profiling() {
    if PROFILING {
        let server_addr = format!("http://127.0.0.1:{}", puffin_http::DEFAULT_PORT);
        match puffin_http::Server::new(&server_addr) {
            Ok(_) => eprintln!("Run this to view profiling data: puffin_viewer {server_addr}"),
            Err(_) => eprintln!("Unable to run the profiling server"),
        }
        eprintln!("Run this to view profiling data: puffin_viewer {server_addr}");
        puffin::set_scopes_on(true);
    }
}
