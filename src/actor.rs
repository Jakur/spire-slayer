use crate::card::{Effect, EffectPair, Target};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::FromEntropy;
use std::mem;

pub trait Actor: std::fmt::Debug {
    fn set_intent(&mut self);
    fn take_damage(&mut self, raw_damage: i32);
    fn compute_damage(&self, raw_damage: i32) -> i32 {
        raw_damage
    }
    fn add_block(&mut self, block: i32);
    fn add_strength(&mut self, strength: i32);
    fn add_weak(&mut self, weak: i32);
    fn scale_attack(&self, base: i32) -> i32;
}

#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub health: i32,
    pub block: i32,
    pub energy: i32,
}

impl Actor for Player {
    fn take_damage(&mut self, damage: i32) {
        if self.block > 0 {
            let mitigated = damage - self.block;
            self.block -= damage as i32;
            if mitigated > 0 {
                self.health -= damage;
            }
        } else {
            self.health -= damage;
        }
    }
    fn set_intent(&mut self) {
        unimplemented!()
    }
    fn add_block(&mut self, block: i32) {
        self.block += block;
    }
    fn add_strength(&mut self, strength: i32) {
        unimplemented!()
    }
    fn add_weak(&mut self, weak: i32) {
        unimplemented!()
    }
    fn scale_attack(&self, base: i32) -> i32 {
        base // Todo scale
    }
}

#[derive(Debug, Clone)]
pub struct JawWorm {
    pub health: i32,
    pub block: i32,
    pub strength: i32,
    pub weak: i32,
    pub intent: usize,
    pub last_actions: Vec<usize>,
    pub queue: Vec<EffectPair>,
}

impl Actor for JawWorm {
    fn set_intent(&mut self) {
        let mut rng = SmallRng::from_entropy();
        let count = self.last_actions.len();
        let choices = [0, 1, 2];
        let mut probs = [45, 30, 25];
        if count >= 2 {
            if self.last_actions[count - 1] == 2 && self.last_actions[count - 2] == 2 {
                //No bellow
                probs[2] -= 25;
            } else if self.last_actions[count - 1] == 0 && self.last_actions[count - 2] == 0 {
                probs[0] -= 45;
            } else if count >= 3
                && self.last_actions[count - 1] == 1
                && self.last_actions[count - 2] == 1
                && self.last_actions[count - 3] == 1
            {
                probs[1] -= 30;
            }
        }
        let choice = choices
            .choose_weighted(&mut rng, |choice| probs[*choice])
            .unwrap();
        match *choice {
            0 => {
                self.queue.push(pair![Attack, Single, 12]);
            }
            1 => {
                self.queue.push(pair![Attack, Single, 7]);
                self.queue.push(pair![Block, Player, 5]);
            }
            2 => {
                self.queue.push(pair![Strength, Player, 5]);
                self.queue.push(pair![Block, Player, 9]);
            }
            _ => {}
        }
    }
    fn take_damage(&mut self, damage: i32) {
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
    fn add_block(&mut self, block: i32) {
        self.block += block;
    }
    fn add_strength(&mut self, strength: i32) {
        self.strength += strength;
    }
    fn add_weak(&mut self, weak: i32) {
        self.weak += weak;
    }
    fn scale_attack(&self, base: i32) -> i32 {
        let mut scaled = base + self.strength;
        if self.weak > 0 {
            scaled -= scaled / 4 + (scaled % 4 != 0) as i32; // Fast int round up
        }
        scaled
    }
}

impl JawWorm {
    pub fn act(&mut self) -> Vec<EffectPair> {
        let queue = mem::replace(&mut self.queue, Vec::new());
        queue
    }
}
