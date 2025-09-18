use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Stylize},
    text::{Line, ToLine},
    widgets::{Block, Cell, Paragraph, Row, Table},
};

use crate::{
    HandleResult, els, scenes,
    stats::{self, Stat},
    view::{Dims, ElGroup, ElSimp, State},
};

pub const BORDER: u16 = 2;

/// Style the provided widget based on its selection state.
fn style_selected<'a, T: 'a + Stylize<'a, T>>(widget: T, selected: bool) -> T {
    if selected {
        widget.fg(Color::Black).bg(Color::White)
    } else {
        widget
    }
}

/// Displays some simple text.
pub struct TextEl(Box<dyn Fn(&State) -> String>);

impl TextEl {
    pub fn new<F: Fn(&State) -> String + 'static>(f: F) -> Self {
        Self(Box::new(f))
    }
}

impl ElSimp for TextEl {
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
        let text = (self.0)(state);
        let widget = Paragraph::new(text).block(Block::bordered());
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
        Dims::new(Constraint::Min(4), Constraint::Length(4))
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
        let modtext = format_modifier(modifier);
        let paragraph = Paragraph::new(vec![
            Line::from(value.to_string()),
            Line::from(modtext),
        ]);

        let widget = style_selected(paragraph, selected)
            .centered()
            .block(Block::bordered().title(stat.short()));
        frame.render_widget(widget, area);
    }

    fn handle_select(&self, state: &State) -> HandleResult {
        HandleResult::Open(Box::new(scenes::StatModal::new(self.0, state)))
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
            Constraint::Length(state.skills.0.len() as u16 + 2),
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
                let proficiency = skill.proficiency;
                let pstr = if proficiency == stats::Proficiency::Untrained {
                    String::from(" ")
                } else {
                    format!("{proficiency:?}")
                        .chars()
                        .next()
                        .unwrap()
                        .to_string()
                };

                let row = Row::new([
                    Cell::new(skill.name.as_str()),
                    Cell::new(skill.stat.short()),
                    Cell::new(pstr),
                    Cell::new(els::format_modifier(skill.modifier(state))),
                ]);
                style_selected(row, selected == Some(i))
            }),
            [
                Constraint::Fill(1),
                Constraint::Max(3),
                Constraint::Max(1),
                Constraint::Max(3),
            ],
        )
        .block(Block::bordered());
        frame.render_widget(widget, area);
    }

    fn handle_select(&self, state: &State, selected: usize) -> HandleResult {
        if let Some(skill) = state.skills.0.get(selected) {
            let modal = scenes::SkillModal::new(&skill.name, state);
            HandleResult::Open(Box::new(modal))
        } else {
            HandleResult::Default
        }
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

#[derive(Clone)]
pub struct EditorState<T> {
    shared_state: std::rc::Rc<std::cell::Cell<T>>,
}

impl<T: Copy> EditorState<T> {
    pub fn new(initial_value: T) -> Self {
        Self {
            shared_state: std::rc::Rc::new(std::cell::Cell::new(initial_value)),
        }
    }

    pub fn get(&self) -> T {
        self.shared_state.get()
    }

    pub fn set(&self, value: T) {
        self.shared_state.set(value);
    }

    pub fn update(&self, effect: impl FnOnce(T) -> T) {
        self.set(effect(self.get()));
    }
}

pub struct SkillProficiencyEditor {
    skill: String,
    state: EditorState<stats::Proficiency>,
}

impl SkillProficiencyEditor {
    pub fn new(
        skill: &str,
        prof: stats::Proficiency,
    ) -> (EditorState<stats::Proficiency>, Self) {
        let state = EditorState::new(prof);
        (
            state.clone(),
            Self {
                skill: skill.to_string(),
                state,
            },
        )
    }
}

impl ElSimp for SkillProficiencyEditor {
    fn dimensions(&self) -> Dims {
        Dims::new(
            Constraint::Min(self.skill.len() as u16 + BORDER),
            Constraint::Length(stats::Proficiency::ALL.len() as u16 + BORDER),
        )
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &State,
        _selected: bool,
    ) {
        let prof = self.state.get();
        let table = Table::default()
            .rows(stats::Proficiency::ALL.iter().map(|p| {
                style_selected(Row::new([format!("{p:?}")]), *p == prof)
            }))
            .block(Block::bordered().title(self.skill.as_str()));

        frame.render_widget(table, area);
    }
}

pub struct StatEditor {
    stat: Stat,
    state: EditorState<i64>,
}

impl StatEditor {
    pub fn new(stat: Stat, initial_value: i64) -> (EditorState<i64>, Self) {
        let state = EditorState::new(initial_value);
        (state.clone(), Self { stat, state })
    }
}

impl ElSimp for StatEditor {
    fn dimensions(&self) -> Dims {
        Dims::new(
            Constraint::Length(6 + BORDER),
            Constraint::Length(2 + BORDER),
        )
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &State,
        _selected: bool,
    ) {
        let score = self.state.get();
        let modifier = format_modifier(Stat::modifier(score));
        let score = format!("< {score} >");
        let widget = Paragraph::new(vec![score.to_line(), modifier.to_line()])
            .centered()
            .block(
                Block::bordered()
                    .title(self.stat.short())
                    .title_alignment(Alignment::Center),
            );

        frame.render_widget(widget, area);
    }
}

pub fn format_modifier(modifier: i64) -> String {
    if modifier < 0 {
        modifier.to_string()
    } else {
        format!("+{modifier}")
    }
}
