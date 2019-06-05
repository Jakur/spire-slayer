use crate::actor::Actor;
use crate::Battle;
use lazy_static::lazy_static;
use std::collections::HashMap;

macro_rules! new_card {
    ($name:expr, $parent:expr, $($atr:ident = $set:expr),*) => {
        CardTemplate {
            name : $name,
            $($atr : $set,)*
            ..(*$parent).clone() // This syntax moves out of the parent struct
        }
    }
}

#[macro_export]
macro_rules! pair {
    ($eff:ident, $target:ident, $mag:expr) => {
        crate::card::EffectPair::new(Effect::$eff($mag), Target::$target) // Shorthand for effect, target, magnitude construction
    }
}

lazy_static! {
    pub static ref CARDS: Vec<CardTemplate> = {
        let mut v = Vec::new();
        v.push(CardTemplate::new(
            "Defend",
            CardType::Skill,
            vec![pair![Block, Player, 5]],
            1,
            false,
            false,
        ));
        v.push(CardTemplate::new(
            "Neutralize",
            CardType::Attack,
            vec![pair![Attack, Single, 3], pair![Weak, Single, 1]],
            0,
            false,
            false,
        ));
        v.push(CardTemplate::new(
            "Strike",
            CardType::Attack,
            vec![pair![Attack, Single, 6]],
            1,
            false,
            false,
        ));
        v.push(new_card![
            "Survivor",
            &v[0],
            effects = vec![pair![Block, Player, 8], pair![Discard, Player, 1]]
        ]);
        v.push(new_card![
            "Acrobatics",
            &v[0],
            effects = vec![pair![Draw, Player, 3], pair![Discard, Player, 1]]
        ]);
        v
    };
    pub static ref NAMES: HashMap<usize, &'static str> = {
        let mut h = HashMap::new();
        for id in 0..CARDS.len() {
            h.insert(id, CARDS[id].name);
        }
        h
    };
    pub static ref IDS: HashMap<&'static str, usize> = {
        let mut h = HashMap::new();
        for id in 0..CARDS.len() {
            h.insert(CARDS[id].name, id);
        }
        h
    };
}

pub trait Playable: std::fmt::Debug {
    fn play(&self, env: &mut Battle, target_id: Option<usize>);
    fn get_type(&self) -> CardType;
    fn get_name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct EffectPair {
    pub effect: Effect,
    pub target: Target,
}

impl EffectPair {
    pub fn new(effect: Effect, target: Target) -> EffectPair {
        EffectPair { effect, target }
    }
    pub fn new_single(eff: Effect) -> EffectPair {
        Self::new(eff, Target::Single)
    }
    pub fn new_multi(eff: Effect) -> EffectPair {
        Self::new(eff, Target::Multi)
    }
    pub fn new_player(eff: Effect) -> EffectPair {
        Self::new(eff, Target::Player)
    }
}

#[derive(Debug, Clone)]
pub enum Target {
    Player,
    Single,
    Multi,
}

#[derive(Debug)]
pub enum Debuff {
    Weak(i32),
}

#[derive(Clone, Debug)]
pub enum Effect {
    Block(i32),
    Attack(i32),
    Weak(i32),
    Draw(usize),
    Discard(usize),
    Strength(i32),
}

#[derive(Clone, Copy, Debug)]
pub enum CardType {
    Attack,
    Skill,
    Power,
}

#[derive(Clone, Debug)]
pub struct CardTemplate {
    pub name: &'static str,
    ty: CardType,
    pub effects: Vec<EffectPair>,
    base_cost: u32,
    ethereal: bool,
    exhaust: bool,
}

impl CardTemplate {
    pub fn new(
        name: &'static str,
        ty: CardType,
        effects: Vec<EffectPair>,
        base_cost: u32,
        ethereal: bool,
        exhaust: bool,
    ) -> CardTemplate {
        CardTemplate {
            name,
            ty,
            effects,
            base_cost,
            ethereal,
            exhaust,
        }
    }
}
