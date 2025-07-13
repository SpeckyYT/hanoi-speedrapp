use crate::{GameState, HanoiApp};

use super::PlayerKind;

impl HanoiApp {
    pub fn replay_play(&mut self) {
        if let PlayerKind::Replay(ref game, ref mut index) = self.player {
            puffin::profile_function!();
            if let Some((time, from, to)) = game.moves.get(*index) {
                if let GameState::Playing(start) = self.state {
                    if start.elapsed() >= *time {
                        self.hanoi.shift(*from, *to);
                        *index += 1;
                        self.moves += 1;
                        if *index >= game.moves.len() {
                            self.state = GameState::Finished(game.time);
                        }
                    }
                }
            }
        }
    }
}
