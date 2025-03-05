use eframe::egui::Key;

use crate::GameState;

use super::{Play, PlayerKind};

pub const SWIFT_KEYS: &[Key] = &[
    Key::Num1, Key::Num2, Key::Num3,
    Key::Num4, Key::Num5, Key::Num6,
    Key::Num7, Key::Num8, Key::Num9,
];

#[derive(Default)]
pub struct SwiftKeys {}

impl Play for SwiftKeys {
    fn title(&self) -> &'static str { "Swift Keys" }
    fn description(&self) -> &'static str {
        "Press the numbers on the numpad to select a pole, select another pole, and it will move the disk from the first pole to the second."
    }
    fn context_play(&mut self, app: &mut crate::HanoiApp, ctx: &eframe::egui::Context) {
        if matches!(app.state, GameState::Finished(_)) || matches!(app.player, PlayerKind::Replay(_, _)) {
            app.swift_pole = None;
            return;
        }

        ctx.input(|input| {
            SWIFT_KEYS.iter().enumerate().for_each(|(i, k)| {
                let inside_bounds = i < app.hanoi.poles_count;
                let pole_not_empty = app.swift_pole.is_some() || !app.hanoi.poles[i].is_empty();
                let key_pressed = input.key_pressed(*k);

                if inside_bounds && pole_not_empty && key_pressed {
                    app.swift_pole = match app.swift_pole {
                        None => Some(i),
                        Some(from) => {
                            app.full_move(from, i);
                            app.reset_undo();
                            None
                        }
                    }
                }
            });
        });
    }
    fn reset(&mut self, app: &mut crate::HanoiApp) {
        app.swift_pole = None;
    }
}
