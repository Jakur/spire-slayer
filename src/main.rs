#[macro_use]
mod card;
mod actor;
use actor::{Actor, JawWorm};
use card::{CardTemplate, CardType, Effect, Target, CARDS};
use mcts::tree_policy::*;
use mcts::*;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::{FromEntropy, SeedableRng, XorShiftRng};
use std::mem;

#[derive(Clone, PartialEq)]
pub struct Card {
    id: usize,
    cost: i32,
}

impl Card {
    fn new(id: usize, cost: i32) -> Card {
        Card { id, cost }
    }
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}, Cost: {}", CARDS[self.id].name, self.cost)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Play(usize),              // Play card in given slot
    TargetPlay(usize, usize), // Play card in given slot directed at entity x
    Discard(usize),
    EndTurn,
}

#[derive(Clone, Debug)]
pub struct Battle {
    pub hand: Vec<Card>,
    pub draw: Vec<Card>,
    pub discard: Vec<Card>,
    pub exhaust: Vec<Card>,
    pub enemy: JawWorm,
    pub slayer: actor::Player,
    pub queue: Vec<Action>,
}

impl Battle {
    fn new(mut deck: Vec<Card>) -> Battle {
        use rand::seq::index::sample;
        let mut rng = SmallRng::from_entropy();
        let mut hand = Vec::new();
        let mut sample = sample(&mut rng, deck.len(), 5).into_vec();
        sample.sort_unstable();
        for index in sample.into_iter().rev() {
            hand.push(deck.swap_remove(index));
        }
        let mut enemy = JawWorm {
            health: 44,
            block: 0,
            strength: 0,
            intent: 0,
            weak: 0,
            last_actions: Vec::new(),
            queue: Vec::new(),
        };
        enemy.set_intent();
        let slayer = actor::Player {
            health: 100,
            block: 0,
            energy: 3,
        };
        Battle {
            hand,
            draw: deck,
            discard: Vec::new(),
            exhaust: Vec::new(),
            enemy,
            slayer,
            queue: Vec::new(),
        }
    }
    fn draw_cards(&mut self, mut number: usize) {
        use rand::seq::index::sample;
        if self.draw.len() > number {
            let mut counter = 0;
            let mut rng = SmallRng::from_entropy();
            let mut sample = sample(&mut rng, self.draw.len(), number).into_vec();
            sample.sort_unstable();
            for index in sample.into_iter().rev() {
                self.hand.push(self.draw.swap_remove(index - counter));
                counter += 1;
            }
        } else {
            number -= self.draw.len();
            self.hand.append(&mut self.draw);
            if self.discard.len() > 0 {
                mem::swap(&mut self.discard, &mut self.draw);
                return self.draw_cards(number);
            }
        }
    }
    fn end_discard(&mut self) {
        self.discard.append(&mut self.hand);
    }
    pub fn apply_effect(&mut self, eff: &Effect, source_id: usize, target_id: usize) {
        let scaled = match eff {
            Effect::Attack(damage) => {
                if source_id == 0 {
                    self.slayer.scale_attack(*damage)
                } else {
                    self.enemy.scale_attack(*damage)
                }
            }
            _ => 0,
        };
        let target: &mut Actor = {
            if target_id == 0 {
                &mut self.slayer
            } else {
                &mut self.enemy
            }
        };
        match eff {
            Effect::Attack(_) => target.take_damage(scaled),
            Effect::Block(block) => target.add_block(*block),
            Effect::Weak(weak) => target.add_weak(*weak),
            Effect::Strength(strength) => target.add_strength(*strength),
            Effect::Discard(discard) => self.queue.push(Action::Discard(*discard)),
            Effect::Draw(card_count) => self.draw_cards(*card_count),
        }
    }
}

impl GameState for Battle {
    type Move = Action;
    type Player = ();
    type MoveList = Vec<Action>;

    fn current_player(&self) -> <Self as GameState>::Player {
        ()
    }

    fn available_moves(&self) -> <Self as GameState>::MoveList {
        if self.enemy.health <= 0 || self.slayer.health <= 0 {
            return vec![]; //Terminal condition
        }
        let mut actions = Vec::new();
        if self.queue.is_empty() {
            let mut actions = Vec::new();
            let max_cost = self.slayer.energy;
            for (index, _c) in self.hand.iter().filter(|c| c.cost <= max_cost).enumerate() {
                actions.push(Action::Play(index));
            }
            actions.push(Action::EndTurn);
            actions
        } else {
            let head = &self.queue[0];
            match head {
                Action::Discard(_) => {
                    for (index, _c) in self.hand.iter().enumerate() {
                        actions.push(Action::Discard(index));
                    }
                }
                _ => {
                    unimplemented!();
                }
            }
            actions
        }
    }

    fn make_move(&mut self, mov: &<Self as GameState>::Move) {
        match mov {
            Action::Play(card_slot) => {
                let card = self.hand.swap_remove(*card_slot);
                let template: &CardTemplate = &CARDS[card.id];
                let source_id = 0;
                for pair in template.effects.iter() {
                    let target_id = match pair.target {
                        Target::Player => source_id,
                        Target::Single => 1,
                        Target::Multi => unimplemented!(),
                    };
                    self.apply_effect(&pair.effect, source_id, target_id)
                }
                self.slayer.energy -= card.cost;
                self.discard.push(card);
            }
            Action::EndTurn => {
                let opp_actions = self.enemy.act();
                for pair in opp_actions {
                    let source_id = 1; // Todo variable
                    let target_id = match pair.target {
                        Target::Player => source_id,
                        Target::Single => 0,
                        Target::Multi => unimplemented!(),
                    };
                    self.apply_effect(&pair.effect, source_id, target_id);
                }
                self.end_discard();
                self.draw_cards(5);
                self.slayer.energy = 3;
                self.slayer.block = 0;
                self.enemy.block = 0;
                self.enemy.set_intent();
            }
            Action::Discard(slot) => {
                let out = self.hand.swap_remove(*slot);
                self.discard.push(out);
                if let Some(Action::Discard(num)) = self.queue.first() {
                    let res = num - 1;
                    if res > 0 {
                        // Continue discarding
                        self.queue[0] = Action::Discard(res);
                    } else {
                        // Queue should always be small so overhead is okay
                        self.queue.remove(0);
                    }
                } else {
                    panic!("Unknown path into discard action"); // Todo more graceful
                }
            }
            _ => unimplemented!(),
        }
    }
}

struct GameEvaluator;

impl Evaluator<SpireMCTS> for GameEvaluator {
    type StateEvaluation = i64;

    fn evaluate_new_state(
        &self,
        state: &Battle,
        moves: &Vec<Action>,
        _: Option<SearchHandle<SpireMCTS>>,
    ) -> (Vec<()>, i64) {
        (vec![(); moves.len()], state.slayer.health as i64)
    }

    fn evaluate_existing_state(
        &self,
        state: &<SpireMCTS as MCTS>::State,
        existing_evaln: &Self::StateEvaluation,
        handle: SearchHandle<SpireMCTS>,
    ) -> Self::StateEvaluation {
        state.slayer.health as i64
    }

    fn interpret_evaluation_for_player(
        &self,
        evaluation: &Self::StateEvaluation,
        player: &<<SpireMCTS as MCTS>::State as GameState>::Player,
    ) -> i64 {
        *evaluation
    }
}

#[derive(Default)]
struct SpireMCTS;

impl MCTS for SpireMCTS {
    type State = Battle;
    type Eval = GameEvaluator;
    type TreePolicy = MyUCT;
    type NodeData = ();
    type ExtraThreadData = ();
}

pub struct MyUCT {
    pub exploration_constant: f64,
}

impl MyUCT {
    pub fn new(exploration_constant: f64) -> Self {
        Self {
            exploration_constant,
        }
    }
}

impl<Spec: MCTS<TreePolicy = Self>> TreePolicy<Spec> for MyUCT {
    type ThreadLocalData = PolicyRng;
    type MoveEvaluation = ();

    fn choose_child<'a, MoveIter>(
        &self,
        moves: MoveIter,
        mut handle: SearchHandle<Spec>,
    ) -> &'a MoveInfo<Spec>
    where
        MoveIter: Iterator<Item = &'a MoveInfo<Spec>> + Clone,
    {
        let total_visits = moves.clone().map(|x| x.visits()).sum::<u64>();
        let adjusted_total = (total_visits + 1) as f64;
        let ln_adjusted_total = adjusted_total.ln();
        handle
            .thread_local_data()
            .policy_data
            .select_by_key(moves, |mov| {
                let sum_rewards = mov.sum_rewards();
                let child_visits = mov.visits();
                // http://mcts.ai/pubs/mcts-survey-master.pdf
                if child_visits == 0 {
                    std::f64::INFINITY
                } else {
                    let explore_term = 2.0 * (ln_adjusted_total / child_visits as f64).sqrt();
                    let mean_action_value = sum_rewards as f64 / child_visits as f64;
                    self.exploration_constant * explore_term + mean_action_value
                }
            })
            .unwrap()
    }
}

#[derive(Clone)]
pub struct PolicyRng {
    rng: XorShiftRng,
}

impl PolicyRng {
    pub fn new() -> Self {
        let rng = SeedableRng::from_seed([1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4]);
        Self { rng }
    }

    pub fn select_by_key<T, Iter, KeyFn>(&mut self, elts: Iter, mut key_fn: KeyFn) -> Option<T>
    where
        Iter: Iterator<Item = T>,
        KeyFn: FnMut(&T) -> f64,
    {
        let mut choice = None;
        let mut num_optimal: u32 = 0;
        let mut best_so_far: f64 = std::f64::NEG_INFINITY;
        for elt in elts {
            let score = key_fn(&elt);
            if score > best_so_far {
                choice = Some(elt);
                num_optimal = 1;
                best_so_far = score;
            } else if score == best_so_far {
                num_optimal += 1;
                if self.rng.gen_bool(1.0 / num_optimal as f64) {
                    choice = Some(elt);
                }
            }
        }
        choice
    }
}

impl Default for PolicyRng {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let game = Battle::new(vec![
        Card::new(1, 1),
        Card::new(1, 1),
        Card::new(0, 1),
        Card::new(0, 1),
        Card::new(0, 1),
    ]);
    let mut mcts = MCTSManager::new(game, SpireMCTS, GameEvaluator, MyUCT::new(50.0));
    mcts.playout_n_parallel(10000, 4);
    mcts.tree().debug_moves();
    //dbg!(mcts.principal_variation(5));
    //dbg!(mcts.principal_variation_states(5));
    //let root = mcts.tree().root_node();
    //    for mov in root.moves() {
    //        dbg!(mov);
    //        let adjusted_total = 2000 as f64;
    //        let ln_adjusted_total = (2001 as f64).ln();
    //        let sum_rewards = mov.sum_rewards();
    //        let child_visits = mov.visits();
    //        // http://mcts.ai/pubs/mcts-survey-master.pdf
    //        let explore_term = if child_visits == 0 {
    //            std::f64::INFINITY
    //        } else {
    //            2.0 * (ln_adjusted_total / child_visits as f64).sqrt()
    //        };
    //        let mean_action_value = sum_rewards as f64 / adjusted_total;
    //        println!("{}", mean_action_value);
    //        println!("{}", 50.0 * explore_term + mean_action_value)
    //    }
    //let example = card::Strike::new(1);
    //println!("{}", std::mem::size_of_val(&example));
    println!("Hello, world!");
}
