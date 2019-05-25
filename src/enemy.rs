use rand::seq::SliceRandom;
use rand::rngs::SmallRng;
use rand::FromEntropy;
use std::mem;
use crate::card::Effect;

#[derive(Debug, Clone)]
pub struct JawWorm {
    pub health: i32,
    pub block: i32,
    pub strength: i32,
    pub intent: usize,
    pub last_actions: Vec<usize>,
    pub queue: Vec<Effect>
}

impl JawWorm {
    pub fn set_intent(&mut self) {
        let mut rng = SmallRng::from_entropy();
        let count = self.last_actions.len();
        let choices = [0, 1, 2];
        let mut probs = [45, 30, 25];
        if count >= 2 {
            if self.last_actions[count-1] == 2 && self.last_actions[count-2] == 2 { //No bellow
                probs[2] -= 25;
            } else if self.last_actions[count-1] == 0 && self.last_actions[count-2] == 0 {
                probs[0] -= 45;
            } else if count >= 3 && self.last_actions[count-1] == 1 && self.last_actions[count-2] == 1 &&
                self.last_actions[count-3] == 1 {
                probs[1] -= 30;
            }
        }
        let choice = choices.choose_weighted(&mut rng, |choice| probs[*choice]).unwrap();
        match *choice {
            0 => {
                self.queue.push(Effect::Attack(12));
            },
            1 => {
                self.queue.push(Effect::Attack(7));
                self.queue.push(Effect::Block(5));
            },
            2 => {
                self.queue.push(Effect::Strength(5));
                self.queue.push(Effect::Block(9));
            }
            _ => {},
        }
    }
    pub fn act(&mut self) -> Vec<Effect> {
        let queue = mem::replace(&mut self.queue, Vec::new());
        queue
    }
    pub fn take_damage(&mut self, damage: i32) {
        if self.block > 0 {
            let mitigated = damage - self.block;
            self.block -= damage;
            if mitigated > 0 {
                self.health -= damage;
            }
        } else {
            self.health -= damage;
        }
    }
    pub fn add_block(&mut self, block: i32) {
        self.block += block;
    }
}