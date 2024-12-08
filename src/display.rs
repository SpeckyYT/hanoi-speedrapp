use std::{fmt::Debug, sync::Arc, time::{Duration, Instant}};

use eframe::{egui::{self, mutex::Mutex, vec2, Align, Align2, CentralPanel, Color32, ComboBox, Direction, DragValue, Event, FontId, Key, Layout, Response, RichText, Sense, Slider, TopBottomPanel, Ui, Vec2, Window}, emath::Numeric};
use egui_dnd::Dnd;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Bar, BarChart};
use indoc::formatdoc;
use once_cell::sync::Lazy;
use pretty_duration::pretty_duration;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use themes::draw_share_tower;

use crate::{hanoi::{RequiredMoves, MAX_DISKS, MAX_DISKS_NORMAL, MAX_POLES, MAX_POLES_NORMAL}, play::PlayerKind, GameState, HanoiApp, APP_NAME};

pub mod themes;

const DISK_HEIGHT: f32 = 30.0;
const DISK_WIDTH_MIN: f32 = 20.0;
const POLE_WIDTH: f32 = 3.0;
const POLE_COLOR: Color32 = Color32::WHITE;
const TEXT_COLOR: Color32 = Color32::WHITE;
const TEXT_OUTLINE_COLOR: Color32 = Color32::BLACK;
const SHARE_BUTTON_DURATION: Duration = Duration::from_millis(1000);
const DEFAULT_QUICK_KEY: (Key, usize, usize) = (Key::Space, 1, 2);

static DEFAULT_HANOI_APP: Lazy<HanoiApp> = Lazy::new(|| {
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

macro_rules! check_changed {
    ($action:expr; $($resp:expr;)*) => {
        if [$(
            $resp.changed,
        )*]
        .iter()
        .any(|&v| v) {
            $action;
        };
    };
}

impl HanoiApp {
    pub fn draw_top_bar(&mut self, ctx: &egui::Context) {
        puffin::profile_function!();

        TopBottomPanel::top("top panel")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(APP_NAME);
                
                ui.separator();

                // put the rest on the right or something

                ui.separator();

                self.share_button(ui);

                ui.vertical(|ui| {
                    self.draw_state(ui);
                });

                ui.separator();
                
                if ui.button(format!("Undo ({:?})", self.undo_key)).clicked() && matches!((&self.player, &self.state), (PlayerKind::Human, GameState::Playing(_))) {
                    self.undo_move();
                }

                if ui.button(format!("Reset ({:?})", self.reset_key)).clicked() {
                    self.soft_reset();
                }

                if ui.button("Settings").clicked() {
                    self.settings_window = !self.settings_window;
                }

                if ui.button("Replays").clicked() {
                    self.replays_window = !self.replays_window;
                }
            });
        });
    }

    pub fn draw_central_panel(&mut self, ctx: &egui::Context) {
        puffin::profile_function!();

        CentralPanel::default()
        .show(ctx, |ui| {
            if self.blindfold && matches!((&self.player, &self.state), (PlayerKind::Human, GameState::Playing(_))) {
                self.draw_blindfold(ui);
            } else {
                self.draw_poles(ui);
            }
            self.draw_windows(ui.ctx());
        });
    }

    pub fn draw_windows(&mut self, ctx: &egui::Context) {
        puffin::profile_function!();

        self.draw_settings_window(ctx);
        self.draw_replays_window(ctx);

        if let GameState::Finished(end) = self.state {
            self.draw_completed_window(ctx, end);
        }
    }

    pub fn draw_blindfold(&self, ui: &mut Ui) {
        ui.centered_and_justified(|ui| {
            ui.heading("[BLINDFOLD ENABLED]");
        });
    }

    pub fn draw_poles(&self, ui: &mut Ui) {
        puffin::profile_function!();

        ui.columns(self.hanoi.poles_count, |uis| {
            uis.iter_mut().enumerate().for_each(|(i, ui)| {
                self.draw_pole(ui, i);
            });
        })
    }

    pub fn draw_pole(&self, ui: &mut Ui, i: usize) {
        puffin::profile_function!();

        ui.with_layout(
            Layout::from_main_dir_and_cross_align(
                match self.poles_position {
                    PolesPosition::Bottom => Direction::BottomUp,
                    PolesPosition::Top => Direction::TopDown,
                },
                Align::Center,
            ),
            |ui| {
                puffin::profile_scope!("pole layout");
                let max_width = ui.available_width();
                let width_step = (max_width - DISK_WIDTH_MIN) / self.hanoi.disks_count as f32;
                let max_height = ui.available_height();
                let spacing = ui.style_mut().spacing.item_spacing.y;
                let disk_height = DISK_HEIGHT.min((max_height - spacing * (self.hanoi.disks_count + 2) as f32) / (self.hanoi.disks_count as f32)).max(0.1);

                self.hanoi.poles[i].iter().for_each(|&disk_number| {
                    puffin::profile_scope!("draw disk");
                    let width = DISK_WIDTH_MIN + width_step * disk_number as f32;
                    let size = Vec2::new(width, disk_height);
                    let (response, painter) = ui.allocate_painter(size, Sense::hover());
                    let color = self.color_theme.to_color(disk_number, self.hanoi.disks_count);
                    painter.rect_filled(response.rect, disk_height / 2.5, color);
                    if self.disk_number {
                        puffin::profile_scope!("disk number");

                        let center_pos = response.rect.center();
                        let align = Align2::CENTER_CENTER;
                        let disk_number = disk_number.to_string();
                        let number_size = disk_height / 1.5;

                        for x in -1..=1 {
                            for y in -1..=1 {
                                if x == 0 || y == 0 { continue }
                                painter.text(
                                    center_pos + vec2(x as f32, y as f32),
                                    align,
                                    &disk_number,
                                    FontId::monospace(number_size),
                                    TEXT_OUTLINE_COLOR,
                                );
                            }
                        }
                        painter.text(
                            center_pos,
                            align,
                            disk_number,
                            FontId::monospace(number_size),
                            TEXT_COLOR,
                        );
                    }
                });

                if self.show_poles {
                    let single_height = disk_height + spacing;
                    let pole_size = self.hanoi.poles[i].len();
                    let remaining_size = self.hanoi.disks_count - pole_size + 1;
                    let remaining_height = remaining_size as f32 * single_height;
                    let size = vec2(POLE_WIDTH, remaining_height);
                    let (response, painter) = ui.allocate_painter(size, Sense::hover());
                    painter.rect_filled(response.rect, 0.0, POLE_COLOR);
                }
            }
        );
    }

    pub fn draw_state(&mut self, ui: &mut egui::Ui) {
        puffin::profile_function!();

        ui.label(match self.state {
            GameState::Reset => "Not started".to_string(),
            GameState::Playing(start) => format!("{:.3?} seconds", start.elapsed().as_secs_f64()),
            GameState::Finished(duration) => {
                let seconds = duration.as_secs_f64();
                let small_time = if seconds < 0.001 { format!("({:?})", duration) } else { "".to_string() };
                format!("{seconds:.3?} seconds {small_time}")
            },
        });
        ui.label(format!("Moves: {}/{} optimal", self.moves, self.hanoi.required_moves()));
    }

    pub fn draw_settings_window(&mut self, ctx: &egui::Context) {
        puffin::profile_function!();
        
        let mut settings_window = self.settings_window;

        Window::new("Settings")
        .open(&mut settings_window)
        .auto_sized()
        .show(ctx, |ui| {
            let max_disks = if self.extra_mode { MAX_DISKS } else { MAX_DISKS_NORMAL };
            let max_poles = if self.extra_mode { MAX_POLES } else { MAX_POLES_NORMAL };
    
            ui.add_enabled_ui(!matches!(self.state, GameState::Playing(_)), |ui| {
                check_changed!(
                    self.soft_reset();
                    ui.add(Slider::new(&mut self.hanoi.disks_count, 1..=max_disks).text("Disks"));
                    {
                        let resp = ui.add(Slider::new(&mut self.hanoi.poles_count, 2..=max_poles).text("Poles"));
                        if resp.changed {
                            self.hanoi.start_pole = self.hanoi.start_pole.min(self.hanoi.poles_count);
                        }
                        resp
                    };
                    ui.add(Slider::new(&mut self.hanoi.start_pole, 1..=self.hanoi.poles_count).text("Start pole"));
                );
    
                let mut any_pole = self.hanoi.end_pole.is_none();
                ui.checkbox(&mut any_pole, "Any end pole");
                if any_pole {
                    self.hanoi.end_pole = None;
                } else {
                    let end_pole = self.hanoi.end_pole.get_or_insert(1);
                    ui.add(Slider::new(end_pole, 1..=self.hanoi.poles_count).text("End pole"));
                };
    
                check_changed!(
                    self.soft_reset();
                    ui.checkbox(&mut self.hanoi.illegal_moves, "Illegal moves");
                    ui.checkbox(&mut self.blindfold, "Blindfold");
                );
            });
            ui.checkbox(&mut self.show_poles, "Show poles");
            ui.checkbox(&mut self.disk_number, "Disk number");

            ui.checkbox(&mut self.reset_on_invalid_move, "Reset on invalid move");

            set_enum_setting(ui, &mut self.color_theme);
            set_enum_setting(ui, &mut self.poles_position);
    
            ui.collapsing("Hotkeys", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Undo");
                    key_input(ui, &mut self.undo_key);
                });
                ui.horizontal(|ui| {
                    ui.label("Reset");
                    key_input(ui, &mut self.reset_key);
                });

                ui.label("Quick keys");
                
                self.quick_keys.retain(|(key, _, _)| !matches!(key, Key::Backspace | Key::Delete));

                Dnd::new(ui, "dnd_quick_keys").show_vec(&mut self.quick_keys, |ui, (key, from, to), handle, _state| {
                    ui.horizontal(|ui| {
                        handle.ui(ui, |ui| {
                            key_input(ui, key);
                            integer_input(ui, from, self.extra_mode);
                            integer_input(ui, to, self.extra_mode);
                        });
                    });
                });

                ui.horizontal(|ui| {
                    if ui.button("+").clicked() && !self.quick_keys.contains(&DEFAULT_QUICK_KEY) {
                        self.quick_keys.push(DEFAULT_QUICK_KEY);
                    }
                    ui.label("Input Del or Backspace in the key input to remove it");
                });
            });

            ui.separator();

            ui.add_enabled_ui(!matches!(self.state, GameState::Playing(_)) && !self.equal_settings(&DEFAULT_HANOI_APP), |ui| {
                if ui.button("Default Settings").double_clicked() {
                    let highscores = self.highscores.clone();
                    *self = (*DEFAULT_HANOI_APP).clone();
                    self.highscores = highscores;
                }
            });
    
            let highscore = self.get_highscores_entry(self.get_current_header()).first();
            if let Some(highscore) = highscore {
                ui.label(format!("Your high score for these settings: {:.3?} seconds", highscore.time.as_secs_f64()));
            } else {
                ui.label("There is no high score for these settings.");
            }
    
            let required_moves = self.hanoi.required_moves();
            let [expert_time_string, computer_time_string] = match required_moves {
                RequiredMoves::Impossible => ["∞".to_string(), "∞".to_string()],
                RequiredMoves::Count(moves) => {
                    let moves = (moves - 1) as f64;
                    let times = [
                        moves / 3.0,
                        moves / 50000000.0,
                    ];
                    times.map(|secs|
                        if secs > Duration::MAX.as_secs_f64() {
                            "∞".to_string()
                        } else {
                            pretty_duration(&Duration::from_secs_f64(secs), None)
                        }
                    )
                }
            };
    
            ui.label(format!("Estimated time for an expert player: {expert_time_string}"));
            ui.label(format!("Estimated time for a computer: {computer_time_string}"));
    
            if matches!(required_moves, RequiredMoves::Impossible) {
                ui.colored_label(Color32::RED, "Warning: Game is impossible. Increase the number of stacks or decrease the number of disks.");
            }
        });

        self.settings_window = settings_window;
    }

    pub fn draw_replays_window(&mut self, ctx: &egui::Context) {
        puffin::profile_function!();
        
        let mut replays_window = self.replays_window;

        Window::new("Replays")
        .open(&mut replays_window)
        .show(ctx, |ui| {
            let max_disks = if self.extra_mode { MAX_DISKS } else { MAX_DISKS_NORMAL };
            let max_poles = if self.extra_mode { MAX_POLES } else { MAX_POLES_NORMAL };
            ui.add(Slider::new(&mut self.replays_filter.disks, 1..=max_disks).text("Disks"));
            {
                let resp = ui.add(Slider::new(&mut self.replays_filter.poles, 2..=max_poles).text("Poles"));
                if resp.changed {
                    self.replays_filter.start_pole = self.replays_filter.start_pole.min(self.replays_filter.poles);
                }
                resp
            };
            ui.add(Slider::new(&mut self.replays_filter.start_pole, 1..=self.replays_filter.poles).text("Start pole"));
            
            let mut any_pole = self.replays_filter.end_pole.is_none();
            ui.checkbox(&mut any_pole, "Any end pole");
            if any_pole {
                self.replays_filter.end_pole = None;
            } else {
                let end_pole = self.replays_filter.end_pole.get_or_insert(1);
                ui.add(Slider::new(end_pole, 1..=self.replays_filter.poles).text("End pole"));
            };

            ui.checkbox(&mut self.replays_filter.illegal_moves, "Illegal moves");
            ui.checkbox(&mut self.replays_filter.blindfold, "Blindfold");

            ui.separator();

            self.draw_highscores_graph(ui);

            ui.separator();

            self.draw_highscores_table(ui);
        });

        self.replays_window = self.replays_window && replays_window;
    } 

    pub fn draw_highscores_graph(&mut self, ui: &mut Ui) {
        puffin::profile_function!();

        egui_plot::Plot::new("highscores_plot")
            .height(128.0)
            .show_axes(false)
            .data_aspect(1.0)
            .show(ui, |plot_ui| plot_ui.bar_chart(BarChart::new(
                match self.highscores.get(&self.replays_filter) {
                    Some(scores) =>
                        scores
                            .iter()
                            .enumerate()
                            .map(|(i, score)| Bar::new((i + 1) as f64, score.time.as_secs_f64()))
                            .collect(),
                    None => vec![],
                }
            )));
    }

    pub fn draw_highscores_table(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        
        match self.highscores.get(&self.replays_filter) {
            Some(games) if !games.is_empty() => {
                let col_def = Column::remainder().resizable(true);

                TableBuilder::new(ui)
                .column(col_def)
                .column(col_def)
                .column(col_def)
                .column(col_def)
                .header(30.0, |mut header| {
                    header.col(|ui| { ui.heading("Time"); });
                    header.col(|ui| { ui.heading("Moves"); });
                    header.col(|ui| { ui.heading("Date"); });
                    header.col(|ui| { ui.heading("Replay"); });
                })
                .body(|body| {
                    body.rows(20.0, games.len(), |mut row| {
                        let index = row.index();
                        let game = &games[index];
                        row.col(|ui| { ui.label(format!("{:.3?}s", game.time.as_secs_f64())); });
                        row.col(|ui| { ui.label(format!("{} moves", game.moves.len())); });
                        row.col(|ui| { ui.label(game.date.format("%Y/%m/%d %H:%M:%S").to_string()); });
                        row.col(|ui| {
                            if ui.button("Replay").clicked() {
                                self.player = PlayerKind::Replay(game.clone(), 0);
                                self.moves = 0;
                                self.hanoi.disks_count = self.replays_filter.disks;
                                self.hanoi.poles_count = self.replays_filter.poles;
                                self.hanoi.start_pole = self.replays_filter.start_pole;
                                self.hanoi.end_pole = self.replays_filter.end_pole;
                                self.hanoi.illegal_moves = self.replays_filter.illegal_moves;
                                self.hanoi.reset();
                                self.state = GameState::Playing(Instant::now());
                            }
                        });
                    });
                });
                
            },
            Some(_) | None => {
                ui.label("No replay with these settings");
            },
        }
    }

    pub fn draw_completed_window(&mut self, ctx: &egui::Context, duration: Duration) {
        puffin::profile_function!();
        
        Window::new("🏆 Game complete!")
        .collapsible(false)
        .auto_sized()
        .show(ctx, |ui| {
            let required_moves = self.hanoi.required_moves().to_number();
            if self.moves <= required_moves {
                ui.label("You had the optimal solution!");
            }
    
            ui.label(format!(
                "Average moves per second: {:.2}",
                self.moves as f64 / duration.as_secs_f64(),
            ));
    
            if self.moves > required_moves {
                ui.label(format!(
                    "Average optimal moves per second: {:.2}",
                    required_moves as f64 / duration.as_secs_f64(),
                ));
            }

            let highscores = self.get_highscores_entry(self.get_current_header());
            
            let highscore = highscores.first()
            .and_then(|first| if first.time == duration {
                highscores.get(1)
            } else {
                Some(first)
            });

            if let Some(highscore) = highscore {
                ui.label(format!("Your best time: {:.3?} seconds", highscore.time.as_secs_f64()));
                if duration > highscore.time {
                    ui.label(format!("High score difference: +{:.3?} seconds", (duration - highscore.time).as_secs_f64()));
                } else {
                    ui.label(RichText::new("New high score!").color(Color32::from_rgb(0xFF, 0xA5, 0x00)));
                    ui.label(format!("Difference: -{:.3?} seconds", (highscore.time - duration).as_secs_f64()));
                }
            }
        });
    }

    fn share_button(&self, ui: &mut Ui) {
        if let GameState::Finished(time) = self.state {
            let required_moves = self.hanoi.required_moves().to_number();
            let is_optimal = self.moves <= required_moves; 

            let time_f64 = time.as_secs_f64();
            
            static LAST_SHARE: Lazy<Arc<Mutex<(u8, Instant)>>> = Lazy::new(|| Arc::new(Mutex::new((0, Instant::now()))));
    
            let button_text = match *LAST_SHARE.lock() {
                (2, instant) if instant.elapsed() < SHARE_BUTTON_DURATION => "Copied to clipboard!",
                (1, instant) if instant.elapsed() < SHARE_BUTTON_DURATION => "Failed to copy...",
                _ => "Share",
            };
    
            if ui.button(button_text).clicked() {
                let time_string = format!("{:.3?}", time_f64);
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
                        time_string.contains("69").then_some("🤣 0 bitches"),
                    ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<&str>>()
                        .join("\n"),
                );
    
                let result = if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    clipboard.set_text(share_text).is_ok()
                } else {
                    false
                } as u8 + 1;
    
                *LAST_SHARE.lock() = (result, Instant::now());
            }
        }
    }    
}

fn key_input(ui: &mut Ui, key: &mut Key) -> Response {
    let mut from_string = format!("{:?}", key);
    let resp = ui.text_edit_singleline(&mut from_string);
    if resp.has_focus() {
        ui.input(|i| for event in &i.events {
            if let Event::Key { key: pkey, ..} = event {
                *key = *pkey;
            }
        })
    }
    resp
}

fn integer_input<T: Numeric>(ui: &mut Ui, input: &mut T, extra_mode: bool) -> Response {
    let resp = ui.add(
        DragValue::new(input)
            .speed(0.0)
            .range(0..=(if extra_mode { MAX_DISKS } else { MAX_DISKS_NORMAL }))
            .clamp_existing_to_range(true)
    );
    resp
}

fn set_enum_setting<T>(ui: &mut Ui, selected: &mut T)
where
    T: IntoEnumIterator + PartialEq + Copy + Debug + 'static,
{
    let type_string = std::any::type_name::<T>();
    ComboBox::from_label(type_string.split("::").last().unwrap_or(type_string))
        .selected_text(format!("{:?}", selected))
        .show_ui(ui, |ui| {
            for mode in T::iter() {
                ui.selectable_value(selected, mode, format!("{:?}", mode));
            }
        });
}
