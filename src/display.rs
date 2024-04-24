use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ComboBox, SidePanel, Slider};
use once_cell::sync::Lazy;
use pretty_duration::pretty_duration;

use crate::{hanoi::{RequiredMoves, MAX_DISKS, MAX_DISKS_NORMAL, MAX_POLES, MAX_POLES_NORMAL}, GameState, HanoiApp, PlayerKind};

static DEFAULT_HANOI_APP: Lazy<HanoiApp> = Lazy::new(|| {
    let mut hanoi_app = HanoiApp::default();
    hanoi_app.hanoi.reset();
    hanoi_app
});

impl HanoiApp {
    pub fn draw_blindfold(&self, ctx: &egui::Context) {
        SidePanel::right("blindfold")
        .show(ctx, |ui| {
            ui.label("[blindfold enabled]");
        });
    }

    pub fn draw_poles(&self, ctx: &egui::Context) {
        for i in (0..self.hanoi.poles_count).rev() {
            SidePanel::right(format!("tower_{i}"))
            .show(ctx, |ui| {
                if self.blindfold {
                    ui.label("[blindfold enabled]");
                } else {
                    for j in 0..self.hanoi.poles[i].len() {
                        let disk = self.hanoi.poles[i][j];
                        ui.label(disk.to_string());
                    }
                }
            });
        }
    }

    pub fn draw_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");

        let max_disks = if self.extra_mode { MAX_DISKS } else { MAX_DISKS_NORMAL };
        let max_poles = if self.extra_mode { MAX_POLES } else { MAX_POLES_NORMAL };

        ui.add_enabled_ui(self.state != GameState::Playing, |ui| {
            ui.add(Slider::new(&mut self.hanoi.disks_count, 1..=max_disks).text("Disks"));
            ui.add(Slider::new(&mut self.hanoi.poles_count, 2..=max_poles).text("Poles"));
            ui.add(Slider::new(&mut self.hanoi.start_pole, 1..=self.hanoi.poles_count).text("Start pole"));

            let mut any_pole = self.hanoi.end_pole.is_none();
            ui.checkbox(&mut any_pole, "Any end pole");
            if any_pole {
                self.hanoi.end_pole = None;
            } else {
                let end_pole = self.hanoi.end_pole.get_or_insert(1);
                ui.add(Slider::new(end_pole, 1..=self.hanoi.poles_count).text("End pole"));
            }

            ui.checkbox(&mut self.hanoi.illegal_moves, "Illegal moves");
            ui.checkbox(&mut self.blindfold, "Blindfold");
        });
        ui.checkbox(&mut self.show_poles, "Show poles");
        ui.checkbox(&mut self.disk_number, "Disk number");

        ComboBox::new("player_select", "Player select")
        .selected_text(match self.player {
            PlayerKind::Human => "Human",
            PlayerKind::Bot => "Bot",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.player, PlayerKind::Human, "Human");
            ui.selectable_value(&mut self.player, PlayerKind::Bot, "Bot");
        });

        ui.add_enabled_ui((self.state != GameState::Playing) && !self.equal_settings(&DEFAULT_HANOI_APP), |ui| {
            if ui.button("Default Settings").clicked() {
                *self = (*DEFAULT_HANOI_APP).clone();
            }
        });

        // TODO:
        ui.label("There is no high score for these settings.");

        let required_moves = self.hanoi.required_moves();
        let infinity_tup = ("∞".to_string(), "∞".to_string());
        let (expert_time_string, computer_time_string) = match required_moves {
            RequiredMoves::Impossible => infinity_tup,
            RequiredMoves::Count(moves) => {
                let expert_seconds = (moves - 1) as f64 / 3.0;
                let computer_seconds = (moves - 1) as f64 / 50000000.0;
                if expert_seconds >= Duration::MAX.as_secs_f64() {
                    infinity_tup
                } else {
                    (
                        pretty_duration(
                            &Duration::from_secs_f64(expert_seconds),
                            None,
                        ),
                        pretty_duration(
                            &Duration::from_secs_f64(computer_seconds),
                            None,
                        ),
                    )
                }

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
            GameState::Playing => format!("{:.3?} seconds", self.start.elapsed().as_secs_f64()),
            GameState::Finished(end) => format!("{:.3?} seconds", end.duration_since(self.start).as_secs_f64()),
        });
        ui.label(format!("Moves: {}/{} optimal", self.moves, self.hanoi.required_moves()));
    }

    pub fn draw_completed(&mut self, ui: &mut egui::Ui, end: Instant) {
        ui.heading("Game complete!");

        let required_moves = self.hanoi.required_moves().to_number();
        if self.moves <= required_moves {
            ui.label("You had the optimal solution!");
        }

        ui.label(format!(
            "Average moves per second: {:.2}",
            self.moves as f64 / end.duration_since(self.start).as_secs_f64(),
        ));

        if self.moves > required_moves {
            ui.label(format!(
                "Average optimal moves per second: {:.2}",
                required_moves as f64 / end.duration_since(self.start).as_secs_f64(),
            ));
        }

        ui.label("Your best time: idk");
        ui.label("High score difference: idk");
    }
}
