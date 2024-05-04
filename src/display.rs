use std::{fmt::Debug, time::Duration};

use eframe::{egui::{self, Align, Align2, Color32, ComboBox, Direction, FontId, Layout, RichText, Sense, SidePanel, Slider, Ui, Vec2}, epaint::Hsva};
use once_cell::sync::Lazy;
use pretty_duration::pretty_duration;
use strum::IntoEnumIterator;

use crate::{hanoi::{RequiredMoves, MAX_DISKS, MAX_DISKS_NORMAL, MAX_POLES, MAX_POLES_NORMAL}, ColorTheme, GameState, HanoiApp, PolesPosition};

const TOWERS_PANEL_ID: &str = "towers";

static DEFAULT_HANOI_APP: Lazy<HanoiApp> = Lazy::new(|| {
    let mut hanoi_app = HanoiApp::default();
    hanoi_app.soft_reset();
    hanoi_app
});

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
    pub fn draw_blindfold(&self, ctx: &egui::Context) {
        SidePanel::right(TOWERS_PANEL_ID)
        .show(ctx, |ui| {
            ui.label("[blindfold enabled]");
        });
    }

    pub fn draw_poles(&self, ctx: &egui::Context) {
        SidePanel::right(TOWERS_PANEL_ID)
        .show(ctx, |ui| {
            ui.columns(self.hanoi.poles_count, |uis| {
                uis.iter_mut().enumerate().for_each(|(i, ui)| {
                    ui.with_layout(
                        Layout::from_main_dir_and_cross_align(
                            match self.poles_position {
                                PolesPosition::Bottom => Direction::BottomUp,
                                PolesPosition::Top => Direction::TopDown,
                            },
                            Align::Center,
                        ),
                        |ui| {
                            for j in 0..self.hanoi.poles[i].len() {
                                let disk_number = self.hanoi.poles[i][j];
                                let width = 20.0 + 10.0 * disk_number as f32;
                                let height = 20.0;
                                let size = Vec2::new(width, height);
                                let (response, painter) = ui.allocate_painter(size, Sense::click_and_drag());
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
                                painter.rect_filled(response.rect, height / 2.5, color);
                                if self.disk_number {
                                    painter.text(response.rect.center(), Align2::CENTER_CENTER, disk_number.to_string(), FontId::monospace(height / 1.5), Color32::BLACK);
                                }
                            }
                        }
                    );
                });
            })
        });
    }

    pub fn draw_settings(&mut self, ui: &mut egui::Ui) {
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

    pub fn draw_completed(&mut self, ui: &mut egui::Ui, duration: Duration) {
        ui.heading("Game complete!");

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
            }
        }
    }
}

fn set_enum_setting<T>(ui: &mut Ui, selected: &mut T)
where
    T: IntoEnumIterator + PartialEq + Copy + Debug + 'static,
{
    let type_string = std::any::type_name::<T>();
    ComboBox::from_label(type_string.split("::").nth(1).unwrap_or(type_string))
        .selected_text(format!("{:?}", selected))
        .show_ui(ui, |ui| {
            for mode in T::iter() {
                ui.selectable_value(selected, mode, format!("{:?}", mode));
            }
        });
}
