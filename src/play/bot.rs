use std::time::Instant;

use crate::{GameState, HanoiApp};

impl HanoiApp {
    pub fn bot_play(&mut self) {
        puffin::profile_function!();
        if self.state == GameState::Reset {
            let start_time = Instant::now();
            self.state = GameState::Playing(start_time);
            self.moves = 0;
            fn hanoi_bot(game: &mut HanoiApp, n: usize, from_rod: usize, to_rod: usize, aux_rod: usize) {
                if n > 0 {
                    hanoi_bot(game, n - 1, from_rod, aux_rod, to_rod);
                    if game.hanoi.shift(from_rod, to_rod) {
                        game.moves += 1;
                    }
                    hanoi_bot(game, n - 1, aux_rod, to_rod, from_rod);
                }
            }
            hanoi_bot(
                self,
                self.hanoi.disks_count,
                self.hanoi.start_pole - 1,
                (self.hanoi.end_pole.unwrap_or(self.hanoi.start_pole)) % self.hanoi.poles_count,
                (self.hanoi.end_pole.unwrap_or(self.hanoi.start_pole + 1)) % self.hanoi.poles_count,
            );
            self.state = GameState::Finished(start_time.elapsed());
        }
    }
}
