use std::{fmt::Debug, time::{Duration, Instant}};

use eframe::{egui::{self, Color32, ComboBox, Context, DragValue, Event, Key, Response, RichText, Slider, Theme, Ui, Window}, emath::Numeric};
use egui_dnd::Dnd;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Bar, BarChart};
use strum::IntoEnumIterator;

use crate::{display::DEFAULT_HANOI_APP, hanoi::{MAX_DISKS, MAX_DISKS_NORMAL, MAX_POLES, MAX_POLES_NORMAL}, play::{is_human_play_enabled, swift_keys::SWIFT_KEYS, HumanPlay, PlayerKind, HUMAN_PLAY}, util::consistency_score, GameState, HanoiApp};

const DEFAULT_QUICK_KEY: (Key, usize, usize) = (Key::Space, 1, 2);

macro_rules! check_changed {
    ($action:expr; $($resp:expr;)*) => {
        if [$(
            $resp.changed(),
        )*]
        .iter()
        .any(|&v| v) {
            $action;
        };
    };
}

impl HanoiApp {
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
                puffin::profile_scope!("hanoi_settings");

                check_changed!(
                    self.soft_reset();
                    ui.add(Slider::new(&mut self.hanoi.disks_count, 1..=max_disks).text("Disks"));
                    {
                        let resp = ui.add(Slider::new(&mut self.hanoi.poles_count, 2..=max_poles).text("Poles"));
                        if resp.changed() {
                            self.hanoi.start_pole = self.hanoi.start_pole.min(self.hanoi.poles_count);
                        }
                        resp
                    };
                    ui.add(Slider::new(&mut self.hanoi.start_pole, 1..=self.hanoi.poles_count).text("Start pole"));
                );

                ui.add_space(10.0);
    
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

            ui.add_space(10.0);

            let emoji = match ctx.theme() {
                Theme::Dark => '☀',
                Theme::Light => '🌙',
            };
            if ui.button(format!("Change app theme: {emoji}")).clicked() {
                ctx.set_theme(match ctx.theme() {
                    Theme::Dark => Theme::Light,
                    Theme::Light => Theme::Dark,
                });
            }

            set_enum_setting(ui, &mut self.color_theme);
            set_enum_setting(ui, &mut self.poles_position);
    
            ui.add_space(10.0);

            ui.label("Playstyles");
            for (enabled, hp) in &mut *HUMAN_PLAY.lock() {
                ui.checkbox(enabled, hp.title());
            }

            ui.collapsing("Hotkeys", |ui| {
                puffin::profile_scope!("hotkeys_settings");

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
                    puffin::profile_scope!("hotkey_line");
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

            ui.add_space(10.0);

            ui.add_enabled_ui(!matches!(self.state, GameState::Playing(_)) && !self.equal_settings(&DEFAULT_HANOI_APP), |ui| {
                puffin::profile_scope!("default_settings");

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
    
            self.draw_estimated_time(ui);
        });

        self.settings_window = settings_window;
    }

    pub fn draw_input_display_window(&mut self, ctx: &Context) {
        puffin::profile_function!();

        let mut input_display_window = self.input_display_window;

        Window::new("Input display")
            .open(&mut input_display_window)
            .auto_sized()
            .show(ctx, |ui| {
                let (
                    qk, sk,
                    p_down, p_drag,
                    reset, undo
                ) = ctx.input(|i| {
                    (
                        self.quick_keys.iter().map(|&qk| (i.key_down(qk.0), qk)).collect::<Vec<_>>(),
                        SWIFT_KEYS.map(|key| (i.key_down(key), key)),
                        i.pointer.any_down(), i.pointer.is_decidedly_dragging(),
                        i.key_down(self.reset_key),
                        i.key_down(self.undo_key),
                    )
                });

                if is_human_play_enabled(HumanPlay::QuickKeys(Default::default())) {
                    ui.horizontal_wrapped(|ui| {
                        for (pressed, (key, ..)) in qk {
                            input_display_key(ui, key, pressed);
                        }
                    });
                }

                if is_human_play_enabled(HumanPlay::SwiftKeys(Default::default())) {
                    ui.horizontal_wrapped(|ui| {
                        for &(pressed, key) in &sk[..self.hanoi.poles_count.min(sk.len())] {
                            input_display_key(ui, key, pressed);
                        }
                    });
                }

                let click_play = is_human_play_enabled(HumanPlay::ClickPlay(Default::default()));
                let drag_and_drop_play = is_human_play_enabled(HumanPlay::DragAndDrop(Default::default()));
                if click_play || drag_and_drop_play {
                    ui.horizontal_wrapped(|ui| {
                        if click_play {
                            input_display_text(ui, "Click", p_down);
                        }
                        if drag_and_drop_play {
                            input_display_text(ui, "Dragging", p_drag);
                        }
                    });
                }

                input_display_key(ui, self.reset_key, reset);
                input_display_key(ui, self.undo_key, undo);
            });

        self.input_display_window = input_display_window;
    }


    pub fn draw_highscores_graph(&mut self, ui: &mut Ui) {
        const MIN_DIVISIONS: u32 = 25;
        const MAX_DELTA: f64 = 1.0;

        puffin::profile_function!();

        egui_plot::Plot::new("highscores_plot")
            .height(256.0)
            .show_axes(true)
            .allow_scroll(false)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_axis_zoom_drag(false)
            .allow_boxed_zoom(false)
            .x_axis_formatter(|fmt, _| format!("{:.3?}s", fmt.value))
            .show(ui, |plot_ui| plot_ui.bar_chart(BarChart::new(
                "Highschores",
                match self.highscores.get(&self.replays_filter) {
                    Some(scores) => {
                        if let Some(last) = scores.last() {
                            let time_delta = (last.time / MIN_DIVISIONS).as_secs_f64();
                            let delta = time_delta.min(MAX_DELTA);
                            let divisions = (last.time.as_secs_f64() / delta).ceil() as usize;

                            let mut bars: Vec<u32> = vec![0; divisions];
                            let mut scores_iter = scores.iter().peekable();

                            for (i, bar) in bars.iter_mut().enumerate().take(divisions) {
                                while let Some(score) = scores_iter.peek() {
                                    let is_last = i == divisions - 1;
                                    let is_in_time_section = score.time.as_secs_f64() <= i as f64 * delta;
                                    if is_last || is_in_time_section {
                                        *bar += 1;
                                        let _ = scores_iter.next();
                                    } else {
                                        break
                                    }
                                }
                            }

                            bars.into_iter().enumerate()
                                .map(|(i, g)| {
                                    let current_time = i as f64 * delta;
                                    let next_time = (i + 1) as f64 * delta;
                                    Bar::new(current_time, g as f64).name(format!("{current_time:.3?}-{next_time:.3?}")).width(next_time - current_time)
                                })
                                .collect()

                            // TODO: this process above could probably be optimized by only looping once over the scores/bars
                        } else {
                            vec![]
                        }
                    },
                    None => vec![],
                }
            )));
    }

    pub fn draw_highscores_table(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        
        match self.highscores.get(&self.replays_filter) {
            Some(games) if !games.is_empty() => {
                let col_def = Column::remainder().at_least(60.0).resizable(true);

                TableBuilder::new(ui)
                .column(col_def)
                .column(col_def)
                .column(col_def)
                .column(col_def)
                .column(col_def)
                .header(30.0, |mut header| {
                    header.col(|ui| { ui.heading("Time"); });
                    header.col(|ui| { ui.heading("Moves"); });
                    header.col(|ui| { ui.heading("Date"); });
                    header.col(|ui| { ui.heading("Replay"); });
                    header.col(|ui| { ui.heading("Consistency"); });
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
                        // TODO: maybe not calculate the consistency every frame for every run
                        row.col(|ui| { ui.label(format!("{:.3?}%", 100.0 * consistency_score(game.moves.iter().map(|m| m.0)))); });
                    });
                });
                
            },
            Some(_) | None => {
                ui.label("No replay with these settings");
            },
        }
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
                if resp.changed() {
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

    pub fn draw_completed_window(&mut self, ctx: &egui::Context, duration: Duration) {
        puffin::profile_function!();
        
        Window::new("🏆 Game complete!")
        .collapsible(false)
        .auto_sized()
        .show(ctx, |ui| {
            ui.heading(format!("{duration:.3?}"));

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
}

fn key_input(ui: &mut Ui, key: &mut Key) -> Response {
    puffin::profile_function!();
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
    puffin::profile_function!();
    ui.add(
        DragValue::new(input)
            .speed(0.0)
            .range(0..=(if extra_mode { MAX_DISKS } else { MAX_DISKS_NORMAL }))
            .clamp_existing_to_range(true)
    )
}

fn set_enum_setting<T>(ui: &mut Ui, selected: &mut T)
where
    T: IntoEnumIterator + PartialEq + Copy + Debug + 'static,
{
    puffin::profile_function!();
    let type_string = std::any::type_name::<T>();
    ComboBox::from_label(type_string.split("::").last().unwrap_or(type_string))
        .selected_text(format!("{:?}", selected))
        .show_ui(ui, |ui| {
            for mode in T::iter() {
                ui.selectable_value(selected, mode, format!("{:?}", mode));
            }
        });
}

#[inline]
fn input_display_key(ui: &mut Ui, key: Key, highlighted: bool) {
    puffin::profile_function!();
    input_display_text(ui, &format!("{key:?}"), highlighted);
}

#[inline]
fn input_display_text(ui: &mut Ui, text: &str, highlighted: bool) {
    puffin::profile_function!();
    let button = ui.button(text);
    if highlighted {
        button.highlight();
    }
}
