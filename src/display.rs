use std::{fmt::Debug, time::Duration};

use eframe::{egui::{self, vec2, Align, Align2, CentralPanel, Color32, ComboBox, Direction, FontId, Layout, RichText, Sense, Slider, TopBottomPanel, Ui, Vec2, Window}, epaint::Hsva};
use once_cell::sync::Lazy;
use pretty_duration::pretty_duration;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::{hanoi::{RequiredMoves, MAX_DISKS, MAX_DISKS_NORMAL, MAX_POLES, MAX_POLES_NORMAL}, GameState, HanoiApp, APP_NAME};

const DISK_HEIGHT: f32 = 30.0;
const DISK_WIDTH_MIN: f32 = 20.0;
const POLE_WIDTH: f32 = 3.0;
const POLE_COLOR: Color32 = Color32::WHITE;
const TEXT_COLOR: Color32 = Color32::WHITE;
const TEXT_OUTLINE_COLOR: Color32 = Color32::BLACK;

static DEFAULT_HANOI_APP: Lazy<HanoiApp> = Lazy::new(|| {
    let mut hanoi_app = HanoiApp::default();
    hanoi_app.soft_reset();
    hanoi_app
});

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum ColorTheme {
    #[default]
    Rainbow,
    Purple,
}

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
        TopBottomPanel::top("top panel")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(APP_NAME);
                
                ui.separator();
                
                ui.vertical(|ui| {
                    self.draw_state(ui);
                });
                ui.separator();
                
                if ui.button("Settings").clicked() {
                    self.settings_window = !self.settings_window;
                }
            });
        });
    }

    pub fn draw_central_panel(&mut self, ctx: &egui::Context) {
        CentralPanel::default()
        .show(ctx, |ui| {
            if self.blindfold {
                self.draw_blindfold(ui);
            } else {
                self.draw_poles(ui);
            }
            self.draw_windows(ui.ctx());
        });
    }

    pub fn draw_windows(&mut self, ctx: &egui::Context) {
        self.draw_settings_window(ctx);

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
        ui.columns(self.hanoi.poles_count, |uis| {
            uis.iter_mut().enumerate().for_each(|(i, ui)| {
                self.draw_pole(ui, i);
            });
        })
    }

    pub fn draw_pole(&self, ui: &mut Ui, i: usize) {
        ui.with_layout(
            Layout::from_main_dir_and_cross_align(
                match self.poles_position {
                    PolesPosition::Bottom => Direction::BottomUp,
                    PolesPosition::Top => Direction::TopDown,
                },
                Align::Center,
            ),
            |ui| {
                let max_width = ui.available_width();
                let width_step = (max_width - DISK_WIDTH_MIN) / self.hanoi.disks_count as f32;
                let max_height = ui.available_height();
                let spacing = ui.style_mut().spacing.item_spacing.y;
                let disk_height = DISK_HEIGHT.min((max_height - spacing * (self.hanoi.disks_count + 2) as f32) / (self.hanoi.disks_count as f32)).max(0.1);

                self.hanoi.poles[i].iter().for_each(|&disk_number| {
                    let width = DISK_WIDTH_MIN + width_step * disk_number as f32;
                    let size = Vec2::new(width, disk_height);
                    let (response, painter) = ui.allocate_painter(size, Sense::hover());
                    let color = match self.color_theme {
                        ColorTheme::Rainbow => {
                            let hsv = Hsva::new(disk_number as f32 / self.hanoi.disks_count as f32, 1.0, 1.0, 1.0);
                            let [ r, g, b ] = hsv.to_srgb();
                            Color32::from_rgb(r, g, b)
                        },
                        ColorTheme::Purple => {
                            if disk_number % 2 == 0 {
                                Color32::from_rgb(212, 156, 234)
                            } else {
                                Color32::from_rgb(134, 88, 154)
                            }
                        },
                    };
                    painter.rect_filled(response.rect, disk_height / 2.5, color);
                    if self.disk_number {
                        let center_pos = response.rect.center();
                        let align = Align2::CENTER_CENTER;
                        let disk_number = disk_number.to_string();
                        let font = FontId::monospace(disk_height / 1.5);

                        for x in -1..=1 {
                            for y in -1..=1 {
                                if x == 0 && y == 0 { continue }
                                painter.text(center_pos + vec2(x as f32, y as f32), align, &disk_number, font.clone(), TEXT_OUTLINE_COLOR);
                            }
                        }
                        painter.text(center_pos, align, disk_number, font, TEXT_COLOR);
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
        let mut settings_window = self.settings_window;

        Window::new("Settings")
        .open(&mut settings_window)
        .auto_sized()
        .show(ctx, |ui| {
            ui.heading("Settings");

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
    
                ui.checkbox(&mut self.hanoi.illegal_moves, "Illegal moves");
                ui.checkbox(&mut self.blindfold, "Blindfold");
            });
            ui.checkbox(&mut self.show_poles, "Show poles");
            ui.checkbox(&mut self.disk_number, "Disk number");
    
            set_enum_setting(ui, &mut self.player);
            set_enum_setting(ui, &mut self.color_theme);
            set_enum_setting(ui, &mut self.poles_position);
    
            ui.add_enabled_ui(!matches!(self.state, GameState::Playing(_)) && !self.equal_settings(&DEFAULT_HANOI_APP), |ui| {
                if ui.button("Default Settings").clicked() {
                    *self = (*DEFAULT_HANOI_APP).clone();
                }
            });
    
            let highscore = self.get_highscores_entry(self.get_current_header()).first();
            if let Some(highscore) = highscore {
                ui.label(format!("Your high score for these settings: {:.3?} seconds", highscore.time.as_secs_f64()));
            } else {
                ui.label("There is no high score for these settings.");
            }
    
            let required_moves = self.hanoi.required_moves();
            let infinity = ["∞".to_string(), "∞".to_string()];
            let [expert_time_string, computer_time_string] = match required_moves {
                RequiredMoves::Impossible => infinity,
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

    pub fn draw_completed_window(&mut self, ctx: &egui::Context, duration: Duration) {
        Window::new("Game complete!")
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
    
            if let Some(highscore) = self.get_highscores_entry(self.get_current_header()).first() {
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
