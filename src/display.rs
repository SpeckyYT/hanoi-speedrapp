use std::{fmt::Debug, sync::{Arc, LazyLock}, time::{Duration, Instant}};

use eframe::egui::{self, mutex::Mutex, panel::Side, CentralPanel, Color32, Layout, RichText, SidePanel, Sides, TextStyle, TopBottomPanel, Ui};
use indoc::formatdoc;
use pretty_duration::pretty_duration;
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use themes::draw_share_tower;

use crate::{get_cursor_position, hanoi::RequiredMoves, play::{PlayerKind, HUMAN_PLAY}, GameState, HanoiApp, APP_NAME};

pub mod themes;
pub mod poles;
pub mod windows;

const DISK_HEIGHT: f32 = 30.0;
const DISK_WIDTH_MIN: f32 = 20.0;
const POLE_WIDTH: f32 = 3.0;
const POLE_COLOR: Color32 = Color32::WHITE;
const TEXT_COLOR: Color32 = Color32::WHITE;
const TEXT_OUTLINE_COLOR: Color32 = Color32::BLACK;
const SHARE_BUTTON_DURATION: Duration = Duration::from_millis(1000);

const TIME_ESTIMATIONS: &[(&str, f64)] = &[
    ("an expert physical player", 3.0),
    ("a good virtual player", 6.0),
    ("a really good virtual player", 9.0),
    ("an expert good virtual player", 12.0),
    ("a computer", 50000000.0),
];

static DEFAULT_HANOI_APP: LazyLock<HanoiApp> = LazyLock::new(|| {
    let mut hanoi_app = HanoiApp::default();
    hanoi_app.soft_reset();
    hanoi_app
});

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum PolesPosition {
    #[default]
    Bottom,
    Top,
}

impl HanoiApp {
    pub fn draw_top_bar(&mut self, ctx: &egui::Context) {
        puffin::profile_function!();

        TopBottomPanel::top("top panel")
        .show(ctx, |ui| {
            Sides::new().show(
                ui,
                |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(APP_NAME);
                    });
                },
                |ui| {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Infos").clicked() {
                            self.infos_panel = !self.infos_panel;
                        }

                        if ui.button("Input display").clicked() {
                            self.input_display_window = !self.input_display_window;
                        }

                        if ui.button("Replays").clicked() {
                            self.replays_window = !self.replays_window;
                        }

                        if ui.button("Settings").clicked() {
                            self.settings_window = !self.settings_window;
                        }

                        if ui.button(format!("Reset ({:?})", self.reset_key)).clicked() {
                            self.soft_reset();
                        }

                        if ui.button(format!("Undo ({:?})", self.undo_key)).clicked() && matches!((&self.player, &self.state), (PlayerKind::Human, GameState::Playing(_))) {
                            self.undo_move();
                        }

                        ui.separator();
                        
                        self.draw_state(ui);
                        
                        self.share_button(ui);
                    });
                },
            );
        });
    }

    pub fn draw_central_panel(&mut self, ctx: &egui::Context) {
        puffin::profile_function!();

        let pointer_pos = get_cursor_position(ctx);

        CentralPanel::default()
        .show(ctx, |ui| {
            if self.blindfold && matches!((&self.player, &self.state), (PlayerKind::Human, GameState::Playing(_))) {
                self.draw_blindfold(ui);
            } else {
                let poles = self.draw_poles(ui, pointer_pos);
                (*HUMAN_PLAY).lock().iter_mut().filter(|(e,_)| *e).for_each(|(_, p)| p.poles_play(self, &poles, pointer_pos));
                self.draw_dragging_disk(ui);
                self.draw_swift_disk(ui);
            }
            self.draw_windows(ui.ctx());
        });
    }

    pub fn draw_windows(&mut self, ctx: &egui::Context) {
        puffin::profile_function!();

        self.draw_settings_window(ctx);
        self.draw_replays_window(ctx);
        self.draw_input_display_window(ctx);

        if let GameState::Finished(end) = self.state {
            self.draw_completed_window(ctx, end);
        }
    }

    pub fn draw_blindfold(&self, ui: &mut Ui) {
        ui.centered_and_justified(|ui| {
            ui.heading("[BLINDFOLD ENABLED]");
        });
    }

    pub fn draw_state(&mut self, ui: &mut egui::Ui) {
        puffin::profile_function!();

        let mut state = match self.state {
            GameState::Reset|GameState::PreReset => "Not started".to_string(),
            GameState::Playing(start) => format!("{:.3?} seconds", start.elapsed().as_secs_f64()),
            GameState::Finished(duration) => {
                let seconds = duration.as_secs_f64();
                let small_time = if seconds < 0.001 { format!("({:?})", duration) } else { "".to_string() };
                format!("{seconds:.3?} seconds {small_time}")
            },
        };
        state.push_str(&format!("\nMoves: {}/{} optimal", self.moves, self.hanoi.required_moves()));
        ui.label(state);
    }

    pub fn draw_infos_panel(&mut self, ctx: &egui::Context) {
        if !self.infos_panel { return }

        puffin::profile_function!();

        SidePanel::new(Side::Right, "infos_panel")
            .width_range(200.0..=600.0)
            .default_width(600.0)
            .show(ctx, |ui| {
                let width = ui.fonts(|f|f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' '));
                ui.spacing_mut().item_spacing.x = width;

                ui.vertical_centered(|ui| ui.heading(APP_NAME));
                
                ui.horizontal_wrapped(|ui| {
                    ui.label("This is an app version of");
                    ui.hyperlink_to("Tower of Hanoi,", "https://en.wikipedia.org/wiki/Tower_of_Hanoi");
                    ui.label("where the controls are optimized for speed.");
                });

                // todo: these two should update depending on the settings
                ui.label("Your goal is to move all disks to a different pole.");
                ui.label("You can only move one disk at a time, and you cannot place a larger disk on top of a smaller one.");

                ui.add_space(10.0);

                let human_play = HUMAN_PLAY.lock();
                let playstyles = human_play.iter().filter(|(b,_)| *b);
                let playstyles_count = playstyles.clone().count();
                let english_number = num_to_words::integer_to_en_us(playstyles_count as isize).unwrap_or("zero".to_string());

                ui.label(format!("There are {english_number} ways to control this game:", ));

                ui.add_space(5.0);

                for (_, playstyle) in playstyles {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label(RichText::new(playstyle.title()).strong());
                        ui.label(format!(": {}", playstyle.description()));
                    });
                }

                ui.add_space(10.0);

                ui.horizontal_wrapped(|ui| {
                    ui.label("Join the developer's");
                    ui.hyperlink_to("Discord server", "https://discord.gg/4EecFku");
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label("Join related Tower of Hanoi");
                    ui.hyperlink_to("Discord server", "https://discord.gg/tykwEuuYCt");
                    ui.label("this is where most of the WR discussions happen");
                });

                if ui.button("Close").clicked() {
                    self.infos_panel ^= true;
                }
            });
    }

    pub fn draw_estimated_time(&self, ui: &mut Ui) {
        puffin::profile_function!();

        let required_moves = self.hanoi.required_moves();

        match required_moves {
            RequiredMoves::Impossible => {
                for (label, _) in TIME_ESTIMATIONS {
                    ui.label(format!("Estimated time for {}: ∞", label));
                }
                ui.colored_label(Color32::RED, "Warning: Game is impossible. Increase the number of stacks or decrease the number of disks.");
            }
            RequiredMoves::Count(moves) => {
                let moves = (moves - 1) as f64;
                for (label, speed) in TIME_ESTIMATIONS {
                    let secs = moves / speed;
                    let time_string = if secs > Duration::MAX.as_secs_f64() {
                        "∞".to_string()
                    } else {
                        pretty_duration(&Duration::from_secs_f64(secs), None)
                    };
                    ui.label(format!("Estimated time for {}: {}", label, time_string));
                }
            }
        }
    }

    fn share_button(&self, ui: &mut Ui) {
        if let GameState::Finished(time) = self.state {
            let required_moves = self.hanoi.required_moves().to_number();
            let is_optimal = self.moves <= required_moves; 

            let time_f64 = time.as_secs_f64();

            static LAST_SHARE: LazyLock<Arc<Mutex<Instant>>> = LazyLock::new(|| Arc::new(Mutex::new(Instant::now() - SHARE_BUTTON_DURATION)));

            let button_text = if LAST_SHARE.lock().elapsed() < SHARE_BUTTON_DURATION {
                "Copied to clipboard!"
            } else {
                "Share"
            };

            if ui.button(button_text).clicked() {
                let time_string = format!("{:.3?}", time_f64);
                let time_string_undotted = time_string.chars().filter(|c| *c != '.').collect::<String>();
                let tower_share = draw_share_tower(self.color_theme, self.poles_position);

                let share_text = formatdoc!(
                    "
                        {tower_share}
                        {APP_NAME} Result:
                        🥞 {} disks
                        ⏱️ {} seconds
                        🎲 {}/{} moves
                        🏎️ {:.2?}{} moves/second
                        {}
                    ",
                    self.hanoi.disks_count,
                    time_string,
                    self.moves, required_moves,
                    required_moves as f64 / time_f64, if is_optimal { "" } else { " optimal" }, // yes this is intended
                    [
                        (!is_optimal).then_some(format!("🚗 {:.2?} moves/second", self.moves as f64 / time_f64).as_str()),
                        (self.hanoi.poles_count != 3).then_some(format!("🗼 {} poles", self.hanoi.poles_count).as_str()),
                        is_optimal.then_some("💯 Optimal solution"),
                        self.blindfold.then_some("😎 Blindfolded"),
                        self.hanoi.illegal_moves.then_some("👮 Illegal moves"),
                        (self.quick_keys.len() != self.hanoi.poles_count * (self.hanoi.poles_count - 1))
                            .then_some(format!("⌨️ {} quick keys", self.quick_keys.len()).as_str()),
                        matches!(self.player, PlayerKind::Replay(_, _)).then_some("🎥 Replay"),
                        time_string_undotted.contains("69").then_some("🤣 0 bitches"),
                        time_string_undotted.contains("247").then_some("😱 #247"),
                        time_string_undotted.contains("666").then_some("😈 666"),
                    ]
                        .into_iter()
                        .flatten()
                        .fold(String::new(), |mut a, b| {
                            a += b;
                            a += "\n";
                            a
                        })
                        .trim_end()
                );

                ui.ctx().copy_text(share_text);
    
                *LAST_SHARE.lock() = Instant::now();
            }
        }
    }    
}
