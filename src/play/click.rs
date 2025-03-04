use eframe::egui::{Pos2, Response, Sense};

use crate::{HanoiApp, PolesVec};

use super::Play;

#[derive(Default)]
pub struct ClickPlay {}

impl Play for ClickPlay {
    fn title(&self) -> &'static str { "Click play" }
    fn poles_play(&mut self, app: &mut HanoiApp, poles: &PolesVec<Response>, _pointer_pos: Option<Pos2>) {
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
