pub const MAX_DISKS: usize = 64;
pub const MAX_POLES: usize = 16;

pub const MAX_DISKS_NORMAL: usize = 30;
pub const MAX_POLES_NORMAL: usize = 9;

use std::fmt::Display;

use arrayvec::ArrayVec;
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};

use crate::highscores::Move;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HanoiGame {
    #[serde(skip)]
    pub poles: [ArrayVec<usize, MAX_DISKS>; MAX_POLES],
    pub poles_count: usize,
    pub disks_count: usize,
    pub start_pole: usize,
    pub end_pole: Option<usize>,
    pub illegal_moves: bool,
    #[serde(skip)]
    pub moves_history: Vec<Move>,
}

impl HanoiGame {
    pub fn new() -> Self {
        let mut hanoi = Self {
            poles: Default::default(),

            poles_count: 3,
            disks_count: 5,
            start_pole: 1,
            end_pole: None,
            illegal_moves: false,

            moves_history: Vec::with_capacity(1024),
        };
        hanoi.reset();
        hanoi
    }
    pub fn shift(&mut self, from: usize, to: usize) -> bool {
        if from == to { return false }
        if let Some(&from_last) = self.poles[from].last() {
            if self.illegal_moves || from_last < *self.poles[to].last().unwrap_or(&usize::MAX) {
                let disk = self.poles[from].pop().unwrap();
                self.poles[to].push(disk);
                return true
            }
        }
        false
    }
    pub fn reset(&mut self) {
        self.moves_history.clear();
        self.poles.iter_mut().for_each(|a| a.clear());

        for i in (1..=self.disks_count).rev() {
            self.poles[self.start_pole - 1].push(i);
        }
    }
    pub fn required_moves(&self) -> RequiredMoves {
        let result = frame_stewart(self.disks_count, self.poles_count);

        match result {
            Some(count) => RequiredMoves::Count(count),
            None => RequiredMoves::Impossible,
        }
    }
    pub fn finished(&self) -> bool {
        let end = ArrayVec::from_iter((1..=self.disks_count).rev());

        if let Some(end_pole) = self.end_pole {
            self.poles[end_pole - 1] == end
        } else {
            for i in 0..self.poles_count {
                if self.start_pole - 1 != i && self.poles[i] == end {
                    return true
                }
            }
            false
        }
    }
}

#[cached]
fn frame_stewart(disks: usize, poles: usize) -> Option<u128> {
    match (disks, poles) {
        (0, _) => Some(0),
        (1, p) if p > 1 => Some(1),
        (d, 3) => Some(2_u128.pow(d as u32) - 1),
        (_, p) if p > 3 => {
            let mut min = None;
            for i in 0..disks {
                if let (Some(first), Some(second)) = (frame_stewart(i, poles), frame_stewart(disks - i, poles - 1)) {
                    let moves = 2 * first + second;
                    min = Some(min.map_or(moves, |current| moves.min(current)));
                }
            }
            min
        }
        _ => None,
    }
}

impl Default for HanoiGame {
    fn default() -> Self {
        Self::new()
    }
}

pub enum RequiredMoves {
    Impossible,
    Count(u128),
}

impl Display for RequiredMoves {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            RequiredMoves::Impossible => "∞".to_string(),
            RequiredMoves::Count(moves) => moves.to_string()
        })
    }
}

impl RequiredMoves {
    pub fn to_number(&self) -> u128 {
        match self {
            Self::Impossible => u128::MAX,
            Self::Count(m) => *m,
        }
    }
}
