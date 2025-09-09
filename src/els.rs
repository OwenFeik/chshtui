use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Rect},
    style::{Color, Stylize},
    text::{Line, Span, ToText},
    widgets::{Block, Cell, Paragraph, Row, Table},
};

use crate::{
    HandleResult,
    stats::{self, Stat},
    view::{self, Dims, ElGroup, ElSimp, Scene, State},
};

/// Style the provided widget based on its selection state.
fn style_selected<'a, T: 'a + Stylize<'a, T>>(widget: T, selected: bool) -> T {
    if selected {
        widget.fg(Color::Black).bg(Color::White)
    } else {
        widget
    }
}

/// Element which displays character name.
pub struct NameEl;

impl ElSimp for NameEl {
    fn dimensions(&self) -> Dims {
        Dims::new(Constraint::Fill(1), Constraint::Max(3))
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &State,
        selected: bool,
    ) {
        let widget =
            Paragraph::new(state.name.as_str()).block(Block::bordered());
        frame.render_widget(style_selected(widget, selected), area);
    }
}

/// Element that renders a single statistic with modifier.
pub struct StatEl(Stat);

impl StatEl {
    pub fn new(stat: Stat) -> Self {
        Self(stat)
    }
}

impl ElSimp for StatEl {
    fn dimensions(&self) -> Dims {
        Dims::new(Constraint::Min(4), Constraint::Min(4))
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &State,
        selected: bool,
    ) {
        let stat = self.0;
        let value = state.stats.score(stat);
        let modifier = Stat::modifier(value);

        let modtext = if modifier < 0 {
            modifier.to_string()
        } else {
            format!("+{modifier}")
        };

        let paragraph = Paragraph::new(vec![
            Line::from(value.to_string()),
            Line::from(modtext),
        ]);

        let widget = style_selected(paragraph, selected)
            .centered()
            .block(Block::bordered().title(stat.short()));
        frame.render_widget(widget, area);
    }
}

/// Element that renders a table of all skills present in the state.
pub struct SkillsEl;

impl ElGroup for SkillsEl {
    fn dimensions(&self, state: &State) -> Dims {
        let longest = state
            .skills
            .0
            .iter()
            .map(|s| s.name.len())
            .max()
            .unwrap_or(0);

        // 1 for gap, 4 for proficiency, 2 for border
        let min_width = longest + 1 + 4 + 2;

        Dims::new(
            Constraint::Min(min_width as u16),
            Constraint::Min(state.skills.0.len() as u16 + 2),
        )
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &State,
        selected: Option<usize>,
    ) {
        let widget = Table::new(
            state.skills.0.iter().enumerate().map(|(i, skill)| {
                let row = Row::new([
                    Cell::new(skill.name.as_str()),
                    Cell::new(
                        render_proficiency(skill.proficiency).right_aligned(),
                    ),
                ]);
                style_selected(row, selected == Some(i))
            }),
            [Constraint::Fill(1), Constraint::Min(4)],
        )
        .block(Block::bordered());
        frame.render_widget(widget, area);
    }

    fn child_count(&self, state: &State) -> usize {
        state.skills.0.len()
    }

    fn child_y(&self, area: Rect, _state: &State, selected: usize) -> u16 {
        area.top() + selected as u16 + 1 // For border.
    }

    fn child_at_y(&self, state: &State, y_offset: u16) -> usize {
        let table_index = y_offset as usize + 1;
        table_index.min(state.skills.0.len().saturating_sub(1))
    }
}

pub struct SkillProficiencyEditor {
    skill: String,
    proficiency: stats::Proficiency,
}

impl SkillProficiencyEditor {
    pub fn new(skill: &str, state: &State) -> Self {
        let proficiency = state
            .skills
            .lookup(skill)
            .map(|s| s.proficiency)
            .unwrap_or(stats::Proficiency::Untrained);
        Self {
            skill: skill.to_string(),
            proficiency,
        }
    }
}

impl ElSimp for SkillProficiencyEditor {
    fn dimensions(&self) -> Dims {
        Dims::new(
            Constraint::Min(self.skill.len() as u16 + 2),
            Constraint::Min(4),
        )
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &State,
        _selected: bool,
    ) {
        let widget = Paragraph::new(vec![
            Line::from(self.skill.as_str()).centered(),
            render_proficiency(self.proficiency).centered(),
        ]);

        frame.render_widget(widget, area);
    }
}

fn render_proficiency<'a>(proficiency: stats::Proficiency) -> Line<'a> {
    const LAYOUT: &[stats::Proficiency] = &[
        stats::Proficiency::Trained,
        stats::Proficiency::Expert,
        stats::Proficiency::Master,
        stats::Proficiency::Legendary,
    ];

    let mut spans: Vec<Span> = Vec::new();
    for prof in LAYOUT {
        let c = format!("{:?}", prof).chars().next().unwrap().to_string();
        if *prof == proficiency {
            spans.push(c.bold().to_string().into());
        } else {
            spans.push(c.into());
        }
    }
    Line::default().spans(spans)
}
