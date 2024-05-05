use std::time::Duration;

use eframe::egui::ahash::AHashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::HanoiApp;

pub type Highscores = AHashMap<Header, Vec<Score>>;
pub type Move = (Duration, usize, usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Header {
    pub poles: usize,
    pub disks: usize,
    pub blindfold: bool,
    pub illegal_moves: bool,
    pub start_pole: usize,
    pub end_pole: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Score {
    pub time: Duration,
    #[serde(default)]
    pub date: DateTime<Utc>,
    pub moves: Vec<Move>,
}

impl HanoiApp {
    pub fn get_current_header(&self) -> Header {
        Header {
            poles: self.hanoi.poles_count,
            disks: self.hanoi.disks_count,
            blindfold: self.blindfold,
            illegal_moves: self.hanoi.illegal_moves,
            start_pole: self.hanoi.start_pole,
            end_pole: self.hanoi.end_pole,
        }
    }

    pub fn get_highscores_entry(&mut self, header: Header) -> &mut Vec<Score> {
        self.highscores.entry(header).or_insert(Vec::new())
    }

    pub fn save_score(&mut self, duration: Duration) {
        let header = self.get_current_header();
        let score = Score {
            time: duration,
            date: Utc::now() - duration,
            moves: self.hanoi.moves_history.clone(),
        };

        let entry = self.get_highscores_entry(header);
        if let Some((i, _)) = entry.iter().enumerate().find(|(_,s)| score.time < s.time) {
            entry.insert(i, score.clone());
        } else {
            entry.push(score);
        }
    }
}
