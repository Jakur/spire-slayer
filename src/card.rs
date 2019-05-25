#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub health: i32,
    pub block: i32,
    pub energy: i32,
}

impl Player {
    pub fn take_damage(&mut self, damage: i32) {
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
    pub fn add_block(&mut self, block: i32) {
        self.block += block;
    }
}


pub struct GameState {
    player: Player,
    enemies: Vec<Player>,
}

#[derive(Debug)]
pub enum Target {
    Player,
    Single,
    Multi,
}

#[derive(Debug)]
pub enum Debuff {
    Weak(i32),
}

#[derive(Debug, Clone)]
pub enum Effect {
    Block(i32),
    Attack(i32),
    Weak(i32),
    Discard(i32),
    Strength(i32),
}

#[derive(Clone, Copy, Debug)]
pub enum CardType {
    Attack,
    Skill,
    Power,
}

#[derive(Debug)]
pub struct CardTemplate {
    pub name: &'static str,
    ty: CardType,
    target: Target,
    pub effects: Vec<Effect>,
    base_cost: u32,
    ethereal: bool,
    exhaust: bool,
}

impl CardTemplate {
    pub fn new(name: &'static str, ty: CardType, target: Target, effects: Vec<Effect>, base_cost: u32,
        ethereal: bool, exhaust: bool) -> CardTemplate {
        CardTemplate {
            name,
            ty,
            target,
            effects,
            base_cost,
            ethereal,
            exhaust,
        }
    }
}

/*macro_rules! new_card {

}*/
