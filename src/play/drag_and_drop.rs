use eframe::egui::{Pos2, Response};

use crate::{GameState, HanoiApp, PolesVec};

use super::{Play, PlayerKind};

#[derive(Default)]
pub struct DragAndDrop {}

impl Play for DragAndDrop {
    fn title(&self) -> &'static str { "Drag And Drop" }
    fn description(&self) -> &'static str {
        "Drag and drop the disks by holding your primary mouse button on one pole and releasing it on another."
    }
    fn poles_play(&mut self, app: &mut HanoiApp, poles: &PolesVec<Response>, pointer_pos: Option<Pos2>) {
        puffin::profile_function!();
        if matches!(app.state, GameState::Finished(_)) || matches!(app.player, PlayerKind::Replay(_, _)) {
            app.dragging_pole = None;
            return;
        }

        match app.dragging_pole {
            None => {
                poles.iter().enumerate().for_each(|(i, pole)| {
                    if pole.drag_started() {
                        app.dragging_pole = Some(i);
                    }
                });
            },
            Some(from) => {
                if poles[from].drag_stopped() {
                    if let Some(pointer_position) = pointer_pos {
                        poles.iter().enumerate().for_each(|(to, pole)| {
                            if from != to && pole.rect.contains(pointer_position) {
                                app.full_move(from, to);
                                app.reset_undo();
                            }
                        });
                    }
                    app.dragging_pole = None;
                }
            },
        }
    }
    fn reset(&mut self, app: &mut HanoiApp) {
        app.dragging_pole = None;
    }
}
