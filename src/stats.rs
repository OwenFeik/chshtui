use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stat {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Stat {
    pub const STATS: &[Stat] = &[
        Stat::Strength,
        Stat::Dexterity,
        Stat::Constitution,
        Stat::Intelligence,
        Stat::Wisdom,
        Stat::Charisma,
    ];

    pub fn short(&self) -> String {
        let name = format!("{:?}", self);
        if name.len() < 3 {
            name.clone()
        } else {
            name[0..3].to_uppercase()
        }
    }

    pub fn modifier(value: i8) -> i64 {
        ((value - 10) / 2) as i64
    }
}

pub struct Stats(HashMap<Stat, i8>);

impl Stats {
    pub fn score(&self, stat: Stat) -> i8 {
        self.0.get(&stat).copied().unwrap_or(10)
    }

    pub fn modifier(&self, stat: Stat) -> i64 {
        Stat::modifier(self.score(stat))
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self(HashMap::from([
            (Stat::Strength, 10),
            (Stat::Dexterity, 10),
            (Stat::Constitution, 10),
            (Stat::Intelligence, 10),
            (Stat::Wisdom, 10),
            (Stat::Charisma, 10),
        ]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Proficiency {
    Untrained,
    Trained,
    Expert,
    Master,
    Legendary,
}

pub struct Skill {
    pub name: String,
    stat: Stat,
    pub proficiency: Proficiency,
}

impl Skill {
    fn new(name: &str, stat: Stat) -> Self {
        Self {
            name: name.to_string(),
            stat,
            proficiency: Proficiency::Untrained,
        }
    }
}

pub struct Skills(pub Vec<Skill>);

impl Skills {
    pub fn lookup(&self, name: &str) -> Option<&Skill> {
        self.0.iter().find(|s| s.name == name)
    }
}

impl Default for Skills {
    fn default() -> Self {
        Self(vec![
            Skill::new("Acrobatics", Stat::Dexterity),
            Skill::new("Arcana", Stat::Intelligence),
            Skill::new("Athletics", Stat::Strength),
            Skill::new("Crafting", Stat::Intelligence),
            Skill::new("Deception", Stat::Charisma),
            Skill::new("Diplomacy", Stat::Charisma),
            Skill::new("Intimidation", Stat::Charisma),
            Skill::new("Medicine", Stat::Wisdom),
            Skill::new("Nature", Stat::Wisdom),
            Skill::new("Occultism", Stat::Intelligence),
            Skill::new("Performance", Stat::Charisma),
            Skill::new("Religion", Stat::Wisdom),
            Skill::new("Society", Stat::Intelligence),
            Skill::new("Stealth", Stat::Dexterity),
            Skill::new("Survival", Stat::Wisdom),
            Skill::new("Thievery", Stat::Dexterity),
        ])
    }
}
