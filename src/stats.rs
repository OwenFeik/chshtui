use std::collections::HashMap;

use ratatui::{
    layout::Constraint,
    style::Stylize,
    text::{Line, Span, ToLine},
    widgets::{Block, Cell, Paragraph, Row, Table},
};

use crate::{
    SheetState,
    layout::{self, SceneElement},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Stat {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Stat {
    const STATS: &[Stat] = &[
        Stat::Strength,
        Stat::Dexterity,
        Stat::Constitution,
        Stat::Intelligence,
        Stat::Wisdom,
        Stat::Charisma,
    ];

    fn short(&self) -> String {
        let name = format!("{:?}", self);
        if name.len() < 3 {
            name.clone()
        } else {
            name[0..3].to_uppercase()
        }
    }

    fn modifier(value: i8) -> i64 {
        ((value - 10) / 2) as i64
    }

    fn render(&self, value: i8) -> Paragraph {
        let modifier = Self::modifier(value);
        let modifier = if modifier < 0 {
            modifier.to_string()
        } else {
            format!("+{modifier}")
        };

        Paragraph::new(vec![
            Line::from(value.to_string()),
            Line::from(modifier),
        ])
        .centered()
        .block(Block::bordered().title(self.short()))
    }

    fn element(&self) -> layout::SceneElement<SheetState> {
        let stat = *self;
        layout::SceneElement::new(
            5,                  // Name, borders.
            Constraint::Min(4), // Top border, score, mod, bottom border.
            Box::new(move |frame, area, state| {
                let score = state.stats.score(stat);
                let widget = stat.render(score);
                frame.render_widget(widget, area);
            }),
        )
    }
}

pub struct Stats(HashMap<Stat, i8>);

impl Stats {
    fn score(&self, stat: Stat) -> i8 {
        self.0.get(&stat).copied().unwrap_or(10)
    }

    fn modifier(&self, stat: Stat) -> i64 {
        Stat::modifier(self.score(stat))
    }

    pub fn elements(&self) -> Vec<layout::SceneElement<SheetState>> {
        Stat::STATS.iter().map(Stat::element).collect()
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

#[derive(Debug, PartialEq, Eq)]
enum Proficiency {
    Untrained,
    Trained,
    Expert,
    Master,
    Legendary,
}

impl Proficiency {
    fn render(&self) -> Line {
        const LAYOUT: &[Proficiency] = &[
            Proficiency::Trained,
            Proficiency::Expert,
            Proficiency::Master,
            Proficiency::Legendary,
        ];

        let mut spans: Vec<Span> = Vec::new();
        for prof in LAYOUT {
            let c = format!("{:?}", prof).chars().next().unwrap().to_string();
            if prof == self {
                spans.push(c.bold().to_string().into());
            } else {
                spans.push(c.into());
            }
        }
        Line::default().spans(spans)
    }
}

struct Skill {
    name: String,
    stat: Stat,
    proficiency: Proficiency,
}

impl Skill {
    fn new(name: &str, stat: Stat) -> Self {
        Self {
            name: name.to_string(),
            stat,
            proficiency: Proficiency::Untrained,
        }
    }

    fn element(&self) -> layout::SceneElement<SheetState> {
        let name = self.name.clone();
        layout::SceneElement::new(
            self.name.len() as u16 + 4 + 2, // Name, prof, borders.
            Constraint::Max(1),             // Single table row.
            Box::new(move |frame, area, state| {
                if let Some(skill) = state.skills.lookup(&name) {
                    let widget = Table::new(
                        [Row::new([
                            Cell::new(name.as_str()),
                            Cell::new(
                                skill.proficiency.render().right_aligned(),
                            ),
                        ])],
                        [Constraint::Fill(1), Constraint::Min(4)],
                    );
                    frame.render_widget(widget, area);
                }
            }),
        )
    }
}

pub struct Skills(Vec<Skill>);

impl Skills {
    fn lookup(&self, name: &str) -> Option<&Skill> {
        self.0.iter().find(|s| s.name == name)
    }

    pub fn elements(&self) -> Vec<SceneElement<SheetState>> {
        self.0.iter().map(|skill| skill.element()).collect()
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
