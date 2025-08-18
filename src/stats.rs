use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, Cell, Paragraph, Row, Table},
};

use crate::{IndexedElement, Location};

#[derive(Debug)]
enum Stat {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Stat {
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
}

struct Stats {
    strength: i8,
    dexterity: i8,
    constitution: i8,
    intelligence: i8,
    wisdom: i8,
    charisma: i8,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }
}

#[derive(Default)]
pub struct StatsElement(Stats);

impl IndexedElement for StatsElement {
    fn render_indexed(
        &self,
        frame: &mut Frame,
        area: Rect,
        idx: Option<usize>,
    ) {
        let ss = &self.0;
        let stats = [
            (Stat::Strength, ss.strength),
            (Stat::Dexterity, ss.dexterity),
            (Stat::Constitution, ss.constitution),
            (Stat::Intelligence, ss.intelligence),
            (Stat::Wisdom, ss.wisdom),
            (Stat::Charisma, ss.charisma),
        ];

        let constraints: Vec<Constraint> =
            std::iter::repeat(Constraint::Ratio(1, stats.len() as u32))
                .take(stats.len())
                .collect();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);
        for (i, (stat, score)) in stats.iter().enumerate() {
            let mut widget = stat.render(*score);
            if idx == Some(i) {
                widget = widget.bold();
            }
            frame.render_widget(widget, layout[i]);
        }
    }

    fn location(&self) -> Location {
        Location::Stats
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
}

struct Skills(Vec<Skill>);

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

#[derive(Default)]
pub struct SkillsElement(Skills);

impl IndexedElement for SkillsElement {
    fn render_indexed(
        &self,
        frame: &mut Frame,
        area: Rect,
        idx: Option<usize>,
    ) {
        let rows = self.0.0.iter().enumerate().map(|(i, s)| {
            let row = Row::new([
                Cell::from(s.name.as_str()),
                Cell::from(s.proficiency.render()),
            ]);
            if idx == Some(i) { row.bold() } else { row }
        });
        let table = Table::default()
            .rows(rows)
            .widths(vec![Constraint::Fill(1), Constraint::Max(4)])
            .block(Block::bordered().title("Skills"));
        frame.render_widget(table, area);
    }

    fn location(&self) -> Location {
        Location::Skills
    }
}
