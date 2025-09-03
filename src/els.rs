use ratatui::{
    layout::Constraint,
    style::{Color, Stylize},
    widgets::{Block, Cell, Row, Table},
};

use crate::layout::{Dims, ElGroup, State};

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
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        state: &State,
        selected: Option<usize>,
    ) {
        let widget = Table::new(
            state.skills.0.iter().enumerate().map(|(i, skill)| {
                let row = Row::new([
                    Cell::new(skill.name.as_str()),
                    Cell::new(skill.proficiency.render().right_aligned()),
                ]);
                if selected == Some(i) {
                    row.bg(Color::White).fg(Color::Black)
                } else {
                    row
                }
            }),
            [Constraint::Fill(1), Constraint::Min(4)],
        )
        .block(Block::bordered());
        frame.render_widget(widget, area);
    }

    fn child_count(&self, state: &State) -> usize {
        state.skills.0.len()
    }

    fn child_y(
        &self,
        area: ratatui::prelude::Rect,
        state: &State,
        selected: usize,
    ) -> u16 {
        area.top() + selected as u16 + 1 // For border.
    }
}
