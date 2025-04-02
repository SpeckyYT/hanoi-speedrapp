use eframe::egui::{self, Context, Key, Modifiers, Pos2};

use crate::{play::{PlayerKind, HUMAN_PLAY}, GameState, HanoiApp};

impl HanoiApp {
    #[inline]
    pub fn prereset(&mut self) {
        if let GameState::PreReset = self.state {
            self.soft_reset();
            (*HUMAN_PLAY).lock().iter_mut().for_each(|(_,p)| p.reset(self));
            self.state = GameState::Reset;
        }
    }

    #[inline]
    pub fn soft_reset(&mut self) {
        self.hanoi.reset();
        self.state = GameState::PreReset;
        self.player = PlayerKind::Human;
        self.moves = 0;
    }

    #[inline]
    pub fn equal_settings(&self, other: &Self) -> bool {
        self.hanoi.disks_count == other.hanoi.disks_count
            && self.hanoi.end_pole == other.hanoi.end_pole
            && self.hanoi.illegal_moves == other.hanoi.illegal_moves
            && self.hanoi.poles_count == other.hanoi.poles_count
            && self.hanoi.start_pole == other.hanoi.start_pole
            && self.blindfold == other.blindfold
            && self.show_poles == other.show_poles
            && self.disk_number == other.disk_number
            && self.player == other.player
    }

    #[inline]
    pub fn check_extra_mode(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            let modifiers = i.modifiers.contains(Modifiers::SHIFT|Modifiers::COMMAND|Modifiers::ALT);
            let space = i.key_down(Key::Enter);
            let mouse = i.pointer.primary_down() && i.pointer.secondary_down();

            if modifiers && space && mouse {
                self.extra_mode = true;
            }
        })
    }
}

pub const fn truthy() -> bool { true }
pub const fn falsy() -> bool { false }

pub const fn reset_key() -> Key { Key::R }
pub const fn undo_key() -> Key { Key::Z }

pub fn quick_keys() -> Vec<(Key, usize, usize)> {
    use Key::*;
    vec![
        (D, 1, 2),
        (F, 1, 3),
        (S, 2, 1),
        (L, 2, 3),
        (J, 3, 1),
        (K, 3, 2),
    ]
}

#[inline]
pub fn get_cursor_position(ctx: &Context) -> Option<Pos2> {
    ctx.input(|i| {
        let hover = i.pointer.hover_pos();
        let interact = i.pointer.interact_pos();
        hover.or(interact)
    })
}
