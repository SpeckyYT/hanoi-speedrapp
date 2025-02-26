use eframe::egui::Key;

use super::Play;

pub const SWIFT_KEYS: &[Key] = &[
    Key::Num1, Key::Num2, Key::Num3,
    Key::Num4, Key::Num5, Key::Num6,
    Key::Num7, Key::Num8, Key::Num9,
];

#[derive(Default)]
pub struct SwiftKeys {}

impl Play for SwiftKeys {
    fn context_play(&mut self, app: &mut crate::HanoiApp, ctx: &eframe::egui::Context) {
        ctx.input(|input| {
            SWIFT_KEYS.iter().enumerate().for_each(|(i, k)| {
                if i < app.hanoi.poles_count && input.key_pressed(*k) {
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
