use ratatui::crossterm::event::KeyCode;

use crate::{
    HandleResult, editors, els, stats,
    view::{self, State},
};

pub struct SheetScene {
    layout: view::Layout,
}

impl SheetScene {
    pub fn new() -> Self {
        let mut layout = view::Layout::new();
        stats::Stat::STATS
            .iter()
            .for_each(|s| layout.add_el(Box::new(els::StatEl::new(*s))));
        layout.add_group(Box::new(els::SkillsEl));
        layout.add_column();
        layout.add_el(Box::new(els::TextEl::new(
            "Name",
            &|s| s.name.clone(),
            &|s| {
                Box::new(editors::StringEditorModal::new(
                    "Name",
                    s.name.clone(),
                    Box::new(|value, state| state.name = value),
                ))
            },
        )));
        layout.add_el(Box::new(els::TextEl::new(
            "Level",
            &|s| format!("Level {}", s.level),
            &|s| {
                Box::new(editors::IntEditorModal::new(
                    "Level",
                    s.level,
                    Box::new(|level, state| state.level = level),
                ))
            },
        )));
        Self { layout }
    }
}

impl view::Scene for SheetScene {
    fn layout(&self) -> &view::Layout {
        &self.layout
    }

    fn handle_key_press(
        &mut self,
        _key: KeyCode,
        _state: &mut State,
    ) -> HandleResult {
        HandleResult::Default
    }
}
