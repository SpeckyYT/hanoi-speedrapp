use eframe::egui::{Pos2, Response};

use crate::{GameState, HanoiApp, PolesVec};

use super::{Play, PlayerKind};

#[derive(Default)]
pub struct ClickPlay {}

impl Play for ClickPlay {
    fn title(&self) -> &'static str { "Click play" }
    fn description(&self) -> &'static str {
        "Click on a pole to select it, then click on another pole to move the disk to it."
    }
    fn poles_play(&mut self, app: &mut HanoiApp, poles: &PolesVec<Response>, _pointer_pos: Option<Pos2>) {
        if matches!(app.state, GameState::Finished(_)) || matches!(app.player, PlayerKind::Replay(_, _)) {
            app.swift_pole = None;
            return;
        }

        for (i, pole) in poles.iter().enumerate() {
            if pole.clicked() {
                app.swift_pole = match app.swift_pole {
                    None => Some(i),
                    Some(from) => {
                        app.full_move(from, i);
                        app.reset_undo();
                        None
                    }
                }
            }
        }
    }
}
