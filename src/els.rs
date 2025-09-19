use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Cell, Paragraph, Row, Table},
};

use crate::{
    HandleResult, editors, els,
    stats::{self, Stat},
    view::{Dims, ElGroup, ElSimp, State},
};

pub const BORDER: u16 = 2;

/// Style the provided widget based on its selection state.
pub fn style_selected<'a, T: 'a + Stylize<'a, T>>(
    widget: T,
    selected: bool,
) -> T {
    if selected {
        widget.fg(Color::Black).bg(Color::White)
    } else {
        widget
    }
}

/// Displays some simple text.
pub struct TextEl {
    title: String,
    get: &'static dyn Fn(&State) -> String,
    set: &'static dyn Fn(String, &mut State),
}

impl TextEl {
    pub fn new<G: Fn(&State) -> String, S: Fn(String, &mut State)>(
        title: &str,
        get: &'static G,
        set: &'static S,
    ) -> Self {
        Self {
            title: title.to_string(),
            get,
            set,
        }
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
        let text = (self.get)(state);
        let widget = Paragraph::new(text)
            .block(Block::bordered().title(self.title.as_str()));
        frame.render_widget(style_selected(widget, selected), area);
    }

    fn handle_select(&self, state: &State) -> HandleResult {
        let modal = editors::StringEditorModal::new(
            &self.title,
            (self.get)(state),
            Box::new(|value, state| (self.set)(value, state)),
        );
        HandleResult::Open(Box::new(modal))
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
        HandleResult::Open(Box::new(editors::StatModal::new(self.0, state)))
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
            .unwrap_or(0) as u16;

        // 1 for gap, 4 for proficiency, 2 for border
        let min_width = longest + 1 + 4 + BORDER;

        Dims::new(
            Constraint::Min(min_width),
            Constraint::Length(state.skills.0.len() as u16 + BORDER),
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
            let modal = editors::SkillModal::new(&skill.name, state);
            HandleResult::Open(Box::new(modal))
        } else {
            HandleResult::Default
        }
    }

    fn child_count(&self, state: &State) -> usize {
        state.skills.0.len()
    }

    fn child_y(&self, area: Rect, _state: &State, selected: usize) -> u16 {
        area.top() + selected as u16 + BORDER / 2
    }

    fn child_at_y(&self, state: &State, y_offset: u16) -> usize {
        let table_index = y_offset as usize + 1;
        table_index.min(state.skills.0.len().saturating_sub(1))
    }
}

pub fn format_modifier(modifier: i64) -> String {
    if modifier < 0 {
        modifier.to_string()
    } else {
        format!("+{modifier}")
    }
}
