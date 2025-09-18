use ratatui::crossterm::event::KeyCode;

use crate::{
    HandleResult, els, stats,
    view::{self, ElSimp, Scene, State},
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
        layout.add_el(Box::new(els::TextEl::new(|s| s.name.clone())));
        layout.add_el(Box::new(els::TextEl::new(|s| {
            format!("Level {}", s.level)
        })));
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
