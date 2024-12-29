use std::time::{Duration, Instant};

use arrayvec::ArrayVec;
use chrono::Datelike;
use clap::Parser;
use cli::Cli;
use display::{themes::ColorTheme, PolesPosition};
use eframe::{egui::{self, Key}, App, Frame, HardwareAcceleration, NativeOptions, APP_KEY};
use highscores::{Header, Highscores};
use play::PlayerKind;
use profiling::enable_profiling;
use serde::{Deserialize, Serialize};
use hanoi::{HanoiGame, MAX_POLES};
use serde_with::{serde_as, DefaultOnError};
use util::*;

mod hanoi;
mod cli;
mod display;
mod play;
mod highscores;
mod util;
mod profiling;

const APP_NAME: &str = "Towers of Hanoi - Speedrapp Edition";

type PolesVec<T> = ArrayVec<T, {MAX_POLES}>;

fn main() -> Result<(), eframe::Error> {
    let cli = Cli::parse();
    if cli.profile { enable_profiling() }
    if cli.backup { backup_save() }

    HanoiApp::run(cli)
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum GameState {
    #[serde(skip)]
    Playing(Instant),
    Finished(Duration),
    #[default]
    Reset,
}

// struct QuickKey {
//     key: Key,
//     from: usize,
//     to: usize,
//     id: usize,
// }

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HanoiApp {
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    hanoi: HanoiGame,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    player: PlayerKind,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    state: GameState,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    moves: u128,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    undo_index: usize,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    reset_on_invalid_move: bool,

    // display
    #[serde(default = "falsy")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    blindfold: bool,
    #[serde(default = "truthy")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    show_poles: bool,
    #[serde(default = "falsy")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    disk_number: bool,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    color_theme: ColorTheme,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    poles_position: PolesPosition,

    // input
    #[serde(default = "reset_key")]
    reset_key: Key,
    #[serde(default = "undo_key")]
    undo_key: Key,
    #[serde(default = "quick_keys")]
    quick_keys: Vec<(Key, usize, usize)>,
    #[serde(skip, default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    dragging_pole: Option<usize>,

    // windows
    #[serde(default = "falsy")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    settings_window: bool,
    #[serde(default = "falsy")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    replays_window: bool,

    // other
    #[serde(skip, default = "falsy")]
    extra_mode: bool,

    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    highscores: Highscores,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    replays_filter: Header,
}

impl Default for HanoiApp {
    fn default() -> Self {
        Self {
            hanoi: Default::default(),
            player: Default::default(),
            state: Default::default(),
            moves: 0,
            undo_index: 0,
            reset_on_invalid_move: false,

            blindfold: false,
            show_poles: true,
            disk_number: false,
            color_theme: Default::default(),
            poles_position: Default::default(),

            reset_key: reset_key(),
            undo_key: undo_key(),
            quick_keys: quick_keys(),
            dragging_pole: None,

            settings_window: false,
            replays_window: false,

            extra_mode: false,

            highscores: Default::default(),
            replays_filter: Default::default(),
        }
    }
}

impl HanoiApp {
    pub fn run(cli: Cli) -> Result<(), eframe::Error> {
        let options = NativeOptions {
            hardware_acceleration: HardwareAcceleration::Preferred,
            vsync: cli.vsync,

            persist_window: true,

            ..Default::default()
        };

        eframe::run_native(
            APP_NAME,
            options,
            Box::new(|cc| Ok(
                Box::new(if let Some(storage) = cc.storage {
                    let mut app = eframe::get_value::<HanoiApp>(storage, eframe::APP_KEY).unwrap_or_default();
                    app.soft_reset();
                    app
                } else {
                    HanoiApp::default()
                })
            )),
        )
    }
}

impl App for HanoiApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        puffin::profile_function!();

        self.check_extra_mode(ctx);

        match self.player {
            PlayerKind::Human => self.player_play(ctx),
            PlayerKind::Bot => self.bot_play(),
            PlayerKind::Replay(..) => self.replay_play(),
        };

        ctx.input(|i| {
            if i.key_pressed(self.reset_key) {
                self.soft_reset();
            }
        });

        self.draw_top_bar(ctx);
        self.draw_central_panel(ctx);

        if matches!(self.state, GameState::Playing(_)) {
            ctx.request_repaint();
        }
    }
}

fn backup_save() {
    if let Some(path) = eframe::storage_dir(APP_NAME) {
        let main_file = path.join(format!("{APP_KEY}.ron"));

        for i in 0..1000 {
            let now = chrono::Utc::now();
            let year = now.year();
            let month = now.month();
            let day = now.day();
            let postfix = if i == 0 { "".to_string() } else { format!(" {}", i) };
            let output_file = path.join(format!("{APP_KEY} BACKUP {year}_{month}_{day}{postfix}.ron"));
            if !output_file.exists() && std::fs::copy(&main_file, &output_file).is_ok() {
                break;
            }
        }
    }
}
