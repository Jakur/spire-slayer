use super::CombatEntity;
use lazy_static::lazy_static;
use std::any::Any;
use tinyvec::{array_vec, ArrayVec};

const MAX_TRIGGERS: usize = 4;

lazy_static! {
    static ref BUFFS: Vec<BaseBuff> = {
        use Trigger::*;
        let s = BaseBuff::single;
        let m = BaseBuff::multi;

        vec![
            s("Artifact", OnApplyDebuff, |s, q| {
                s.buff().unwrap().quantity = 0;
                *q -= 1;
            }),
            s("Barricade", OnStartTurn, |_, _| todo!()),
            s("Buffer", OnLoseHp, |s, q| {
                s.damage().unwrap().0 = 0;
                *q -= 1;
            }),
            s("Dexterity", OnCardBlock, |s, q| {
                s.block().unwrap().value += *q;
            }),
            s("Draw Card", OnStartTurn, |_, _| todo!()),
            s("Energized", OnStartTurn, |_, _| todo!()),
            s("Focus", OnActivateOrb, |_, _| todo!()),
            m(
                "Intangible",
                &[
                    (OnTakeDamage, |s, _| s.damage().unwrap().0 = 1),
                    (OnStartTurn, |_s, q| *q -= 1),
                ],
            ),
        ]
    };
}

#[derive(Clone)]
enum Trigger {
    Dummy,
    OnApplyDebuff,
    OnStartTurn,
    OnLoseHp,
    OnCardBlock,
    OnActivateOrb,
    OnTakeDamage,
}

struct State {
    dummy: i32,
    queue: Vec<Box<dyn Any>>,
    buff_manager: BuffManager,
}

impl State {
    fn new(dummy: i32) -> Self {
        State {
            dummy,
            queue: vec![Box::new(Attack { value: 3, times: 4 })],
            buff_manager: BuffManager::new(0),
        }
    }
    fn attack(&mut self) -> Option<&mut Attack> {
        self.queue.last_mut()?.downcast_mut()
    }
    fn block(&mut self) -> Option<&mut Block> {
        self.queue.last_mut()?.downcast_mut()
    }
    fn buff(&mut self) -> Option<&mut Buff> {
        self.queue.last_mut()?.downcast_mut()
    }
    fn damage(&mut self) -> Option<&mut Damage> {
        self.queue.last_mut()?.downcast_mut()
    }
    fn test(&mut self) {
        self.buff_manager.on_action(&mut self);
    }
}

struct Buff {
    base: &'static BaseBuff,
    quantity: i32,
}

impl Buff {
    fn new(base: &'static BaseBuff, quantity: i32) -> Self {
        Self { base, quantity }
    }
    fn debuff(&self) -> bool {
        self.base.debuff || (self.quantity < 0) // Todo is this universal?
    }
}

struct Attack {
    value: i32,
    times: i32,
}

struct Block {
    value: i32,
    times: i32,
}

struct Damage(i32);

pub struct BuffManager {
    entity_index: usize,
    vec: Vec<(&'static BaseBuff, i32)>,
    delete: Vec<usize>,
}

impl BuffManager {
    fn new(entity_index: usize) -> Self {
        Self {
            entity_index,
            vec: Vec::new(),
            delete: Vec::new(),
        }
    }
    fn on_action(&mut self, state: &mut State) {
        for (index, (b, mut q)) in self.vec.iter_mut().enumerate() {
            for e in b.effects.iter() {
                (e.effect)(state, &mut q);
                if q == 0 {
                    self.delete.push(index)
                }
            }
        }
        if self.delete.len() == 1 {
            let index = self.delete.pop().unwrap();
            self.vec.swap_remove(index);
        } else if self.delete.len() > 1 {
            self.delete = Vec::new();
            self.vec.retain(|(_b, q)| *q > 0);
        }
    }
}

struct Effect {
    trigger: Trigger,
    effect: ModifyState,
}

type ModifyState = fn(&mut State, &mut i32);

impl Effect {
    fn new(trigger: Trigger, effect: ModifyState) -> Self {
        Self { trigger, effect }
    }
}

impl std::default::Default for Effect {
    fn default() -> Self {
        Self {
            trigger: Trigger::Dummy,
            effect: |_, _| {},
        }
    }
}

struct BaseBuff {
    name: &'static str,
    effects: ArrayVec<[Effect; MAX_TRIGGERS]>,
    debuff: bool,
}

impl BaseBuff {
    fn single(name: &'static str, trigger: Trigger, effect: ModifyState) -> Self {
        let e = array_vec![[Effect; MAX_TRIGGERS] => Effect::new(trigger, effect)];
        Self {
            name,
            effects: e,
            debuff: false,
        }
    }
    fn multi(name: &'static str, slice: &[(Trigger, ModifyState)]) -> Self {
        let e = slice
            .iter()
            .cloned()
            .map(|(a, b)| Effect::new(a, b))
            .collect();
        Self {
            name,
            effects: e,
            debuff: false,
        }
    }
    fn into_debuff(mut self) -> Self {
        self.debuff = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn foo() {
        let mut s = State::new(5);
        let mut q = 1;
        (BUFFS[0].effects[0].effect)(&mut s, &mut q);
        assert_eq!(s.dummy, 10);
        assert_eq!(q, 0);
        assert_eq!(s.attack().map(|a| a.value), Some(3));
        assert!(s.block().is_none());
    }
}
