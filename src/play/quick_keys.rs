use crate::GameState;

use super::{Play, PlayerKind};

#[derive(Default)]
pub struct QuickKeys {}

impl Play for QuickKeys {
    fn title(&self) -> &'static str { "Quick Keys" }
    fn description(&self) -> &'static str {
        "Press a key on the keyboard to move a disk from one pole to another. Check the `Hotkeys` in the settings."
    }
    fn context_play(&mut self, app: &mut crate::HanoiApp, ctx: &eframe::egui::Context) {
        puffin::profile_function!();
        ctx.input(|i| {
            for qki in 0..app.quick_keys.len() {
                let (key, from, to) = app.quick_keys[qki];
                if i.key_pressed(key) {
                    app.full_move(from - 1, to - 1);
                    app.reset_undo();
                }
            }
            if matches!((&app.player, &app.state), (PlayerKind::Human, GameState::Playing(_))) && i.key_pressed(app.undo_key) {
                app.undo_move();
            }
        });
    }
}
