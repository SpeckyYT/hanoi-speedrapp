use eframe::egui::{self, Key, Modifiers};

use crate::{hanoi::{MAX_DISKS, MAX_DISKS_NORMAL, MAX_POLES, MAX_POLES_NORMAL, MIN_DISKS, MIN_POLES, MIN_POLES_NORMAL}, play::PlayerKind, GameState, HanoiApp};

impl HanoiApp {
    pub fn soft_reset(&mut self) {
        self.hanoi.reset();
        self.state = GameState::Reset;
        self.player = PlayerKind::Human;
        self.moves = 0;
    }

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
    pub const fn disks_and_poles(&self) -> DisksAndPoles {
        disks_and_poles(self.extra_mode)
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

pub struct DisksAndPoles {
    pub min_disks: usize,
    pub max_disks: usize,
    pub min_poles: usize,
    pub max_poles: usize,
}

#[inline]
pub const fn disks_and_poles(extra_mode: bool) -> DisksAndPoles {
    DisksAndPoles {
        min_disks: MIN_DISKS,
        max_disks: if extra_mode { MAX_DISKS } else { MAX_DISKS_NORMAL },

        min_poles: if extra_mode { MIN_POLES } else { MIN_POLES_NORMAL },
        max_poles: if extra_mode { MAX_POLES } else { MAX_POLES_NORMAL },
    }
}
