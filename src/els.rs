use ratatui::{
    Frame,
    layout::{Constraint, Direction, Position, Rect},
    style::{Color, Stylize},
    text::{Line, ToLine},
    widgets::{Block, Cell, Paragraph, Row, Table},
};

use crate::{
    SheetState, editors, els,
    roll::{self, Roll},
    stats::{self, Stat},
    view::{self, Dims, ElGroup, ElSimp, Handler, centre_of},
};

pub const BORDER: u16 = 2;

pub type State = SheetState;

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
    select: &'static dyn Fn(&State) -> Box<dyn view::Scene<State>>,
}

impl TextEl {
    pub fn new<
        G: Fn(&State) -> String,
        S: Fn(&State) -> Box<dyn view::Scene<State>>,
    >(
        title: &str,
        get: &'static G,
        select: &'static S,
    ) -> Self {
        Self {
            title: title.to_string(),
            get,
            select,
        }
    }
}

impl ElSimp<State> for TextEl {
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
        let widget = Paragraph::new(style_selected(text.to_line(), selected))
            .block(Block::bordered().title(self.title.as_str()));
        frame.render_widget(widget, area);
    }

    fn handle_select(&self, state: &State) -> Handler {
        Handler::Open((self.select)(state))
    }
}

/// Element that renders a single statistic with modifier.
pub struct StatEl(Stat);

impl StatEl {
    pub fn new(stat: Stat) -> Self {
        Self(stat)
    }
}

impl ElSimp<State> for StatEl {
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
            style_selected(Line::from(value.to_string()), selected),
            style_selected(Line::from(modtext), selected),
        ]);

        let widget = paragraph
            .centered()
            .block(Block::bordered().title(stat.short()));
        frame.render_widget(widget, area);
    }

    fn handle_select(&self, state: &State) -> Handler {
        Handler::Open(editors::stat_modal(self.0, state))
    }

    fn handle_roll(&self, state: &State) -> Handler {
        let modifier = state.stats.modifier(self.0);
        let modal =
            editors::RollModal::new(Roll::new(1, 20).plus(modifier as f64));
        Handler::Open(Box::new(modal))
    }
}

/// Element that renders a table of all skills present in the state.
pub struct SkillsEl;

impl ElGroup<State> for SkillsEl {
    fn direction(&self) -> Direction {
        Direction::Vertical
    }

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

    fn handle_select(&self, state: &State, selected: usize) -> Handler {
        if let Some(skill) = state.skills.0.get(selected) {
            let modal = editors::SkillModal::new(&skill.name, state);
            Handler::Open(Box::new(modal))
        } else {
            Handler::Default
        }
    }

    fn handle_roll(&self, state: &State, selected: usize) -> Handler {
        if let Some(skill) = state.skills.0.get(selected) {
            let modifier = skill.modifier(state);
            let modal =
                editors::RollModal::new(Roll::new(1, 20).plus(modifier as f64));
            Handler::Open(Box::new(modal))
        } else {
            Handler::Default
        }
    }

    fn child_count(&self, state: &State) -> usize {
        state.skills.0.len()
    }

    fn child_pos(
        &self,
        area: Rect,
        _state: &State,
        selected: usize,
    ) -> (u16, u16) {
        let x = area.x + area.width / 2;
        let y = area.top() + selected as u16 + BORDER / 2;
        (x, y)
    }

    fn child_at_pos(
        &self,
        area: Rect,
        state: &State,
        _x: u16,
        y: u16,
    ) -> usize {
        let y_offset = y - area.y;
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

pub struct RollDisplay {
    dimensions: Dims,
    roll_text: String,
    result_text: String,
}

impl RollDisplay {
    pub fn new(outcome: &roll::RollOutcome) -> Self {
        let roll_text = outcome.format_roll();
        let result_text = format!(
            "{} ({})",
            outcome.format_value(),
            outcome.format_results()
        );
        let width = roll_text.len().max(result_text.len()) as u16;
        let dimensions =
            Dims::new(Constraint::Length(width), Constraint::Length(2));
        Self {
            dimensions,
            roll_text,
            result_text,
        }
    }
}

impl ElSimp<State> for RollDisplay {
    fn dimensions(&self) -> Dims {
        self.dimensions
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &State,
        selected: bool,
    ) {
        let widget = Paragraph::new(vec![
            self.roll_text.to_line(),
            self.result_text.to_line(),
        ])
        .centered();
        frame.render_widget(style_selected(widget, selected), area);
    }
}

pub struct Dice;

impl Dice {
    const DICE: &[u32] = &[4, 6, 8, 10, 12, 20];

    fn iter_layout(
        &self,
        area: Rect,
    ) -> impl Iterator<Item = (usize, Rect, u32)> {
        let areas = ratatui::prelude::Layout::new(
            Direction::Horizontal,
            vec![Constraint::Fill(1); Dice::DICE.len()],
        )
        .split(area)
        .to_vec();
        areas
            .into_iter()
            .zip(Dice::DICE)
            .enumerate()
            .map(|(i, (area, d))| (i, area, *d))
    }
}

impl ElGroup<State> for Dice {
    fn dimensions(&self, _state: &State) -> Dims {
        Dims::new(
            Constraint::Min(4 * Dice::DICE.len() as u16 + BORDER),
            Constraint::Length(BORDER + 1),
        )
    }

    fn direction(&self) -> Direction {
        Direction::Horizontal
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &State,
        selected: Option<usize>,
    ) {
        for (i, area, d) in self.iter_layout(area) {
            let area = if i == Dice::DICE.len() - 1 {
                area
            } else {
                Rect::new(area.x, area.y, area.width + 1, area.height)
            };
            let text = format!("d{d}");
            let widget = Paragraph::new(style_selected(
                text.to_line(),
                selected == Some(i),
            ))
            .block(Block::bordered());
            frame.render_widget(widget, area);
        }
    }

    fn child_count(&self, _state: &State) -> usize {
        Dice::DICE.len()
    }

    fn child_pos(
        &self,
        area: Rect,
        _state: &State,
        selected: usize,
    ) -> (u16, u16) {
        for (i, d_area, _) in self.iter_layout(area) {
            if i == selected {
                return centre_of(d_area);
            }
        }
        centre_of(area)
    }

    fn child_at_pos(
        &self,
        area: Rect,
        _state: &State,
        x: u16,
        y: u16,
    ) -> usize {
        for (i, d_area, _) in self.iter_layout(area) {
            if d_area.contains(Position::new(x, y)) {
                return i;
            }
        }
        0
    }

    fn handle_roll(&self, _state: &State, selected: usize) -> Handler {
        let d = Dice::DICE.get(selected).copied().unwrap_or(20);
        Handler::Open(Box::new(editors::RollModal::new(Roll::new(1, d))))
    }

    fn handle_select(&self, state: &State, selected: usize) -> Handler {
        self.handle_roll(state, selected)
    }
}

pub struct RollHistory {
    max_rolls_to_display: usize,
}

impl RollHistory {
    pub fn new(max_rolls_to_display: usize) -> Self {
        Self {
            max_rolls_to_display,
        }
    }
}

impl ElGroup<State> for RollHistory {
    fn dimensions(&self, state: &State) -> Dims {
        Dims::new(
            Constraint::Fill(1),
            Constraint::Length(
                state.rolls.len().min(self.max_rolls_to_display) as u16
                    + 1 // Header
                    + BORDER,
            ),
        )
    }

    fn direction(&self) -> Direction {
        Direction::Vertical
    }

    fn child_count(&self, state: &State) -> usize {
        state.rolls.len().min(self.max_rolls_to_display)
    }

    fn child_pos(
        &self,
        area: Rect,
        _state: &State,
        selected: usize,
    ) -> (u16, u16) {
        let x = area.x + area.width / 2;
        let y = area.y + 1 + BORDER / 2 + selected as u16;
        (x, y)
    }

    fn child_at_pos(
        &self,
        area: Rect,
        state: &State,
        _x: u16,
        y: u16,
    ) -> usize {
        let y_offset = y.saturating_sub(area.y + 1 + BORDER / 2);
        (y_offset as usize)
            .min(state.rolls.len().min(self.max_rolls_to_display))
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &State,
        selected: Option<usize>,
    ) {
        let rows = state
            .rolls
            .iter()
            .rev()
            .take(self.max_rolls_to_display)
            .enumerate()
            .map(|(i, oc)| {
                let r = Row::new([
                    oc.format_roll(),
                    oc.format_results(),
                    oc.format_value(),
                ]);
                style_selected(r, selected == Some(i))
            });
        let table = Table::default()
            .header(Row::new(["Roll", "Results", "Total"]))
            .rows(rows)
            .block(Block::bordered());
        frame.render_widget(table, area);
    }
}
