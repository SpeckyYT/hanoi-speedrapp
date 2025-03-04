use std::{sync::Arc, time::Instant};

use eframe::egui::{self, mutex::Mutex, Pos2, Response};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{highscores::Score, GameState, HanoiApp, PolesVec};

mod bot;
mod replay;

#[derive(Debug, Default, Clone, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum PlayerKind {
    #[default]
    Human,
    Bot,
    Replay(Score, usize),
}

pub trait Play {
    fn title(&self) -> &'static str;
    fn context_play(&mut self, _app: &mut HanoiApp, _ctx: &egui::Context) {}
    fn poles_play(&mut self, _app: &mut HanoiApp, _poles: &PolesVec<Response>, _pointer_pos: Option<Pos2>) {}
    fn reset(&mut self, _app: &mut HanoiApp) {}
}

macro_rules! human_play {
    ($($mod:ident => $struct:ident,)*) => {
        $(mod $mod;)*

        pub enum HumanPlay {
            $($struct($mod::$struct),)*
        }
        impl HumanPlay {
            pub fn title(&self) -> &'static str {
                match self {
                    $(HumanPlay::$struct(play) => play.title(),)*
                }
            }
            pub fn context_play(&mut self, app: &mut HanoiApp, ctx: &egui::Context) {
                match self {
                    $(HumanPlay::$struct(play) => play.context_play(app, ctx),)*
                }
            }
            pub fn poles_play(&mut self, app: &mut HanoiApp, poles: &PolesVec<Response>, pointer_pos: Option<Pos2>) {
                match self {
                    $(HumanPlay::$struct(play) => play.poles_play(app, poles, pointer_pos),)*
                }
            }
            pub fn reset(&mut self, app: &mut HanoiApp) {
                match self {
                    $(HumanPlay::$struct(play) => play.reset(app),)*
                }
            }
        }
        pub static HUMAN_PLAY: Lazy<Arc<Mutex<[(bool, HumanPlay); [$(stringify!($mod),)*].len()]>>> = Lazy::new(|| Arc::new(Mutex::new(
            [
                $(
                    (true, HumanPlay::$struct($mod::$struct::default())),
                )*
            ]
        )));
    };
}

human_play!{
    quick_keys => QuickKeys,
    swift_keys => SwiftKeys,
    drag_and_drop => DragAndDrop,
    click => ClickPlay,
}

impl HanoiApp {
    pub fn full_move(&mut self, from: usize, to: usize) {
        if !matches!(self.state, GameState::Finished(_)) {
            if self.hanoi.shift(from, to) {
                if self.state == GameState::Reset {
                    self.state = GameState::Playing(Instant::now());
                }
                self.moves += 1;
                if let GameState::Playing(time) = self.state {
                    self.hanoi.moves_history.push((time.elapsed(), from, to));
                }
            } else if self.reset_on_invalid_move {
                self.soft_reset();
            }
        }
    }
    pub fn undo_move(&mut self) {
        if let Some((_, from, to)) = self.undo_index.checked_sub(1).and_then(|i| self.hanoi.moves_history.get(i)) {
            self.full_move(*to, *from);
            self.undo_index -= 1;
        }
    }
    #[inline]
    pub fn reset_undo(&mut self) {
        self.undo_index = self.hanoi.moves_history.len();
    }
}
