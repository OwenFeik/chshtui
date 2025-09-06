use ratatui::crossterm::event::KeyCode;

use crate::{HandleResult, els, stats, view};

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
        layout.add_el(Box::new(els::NameEl));
        Self { layout }
    }
}

impl view::Scene for SheetScene {
    fn layout(&mut self) -> &mut view::Layout {
        &mut self.layout
    }

    fn handle_key_press(&mut self, _key: KeyCode) -> HandleResult {
        HandleResult::Default
    }
}

pub struct SkillModal {
    layout: view::Layout,
    skill: String,
}

impl Scene for SkillModal {
    pub fn new(skill: &str) -> Self {
        let mut layout = view::Layout::new();
        Self {
            layout,
            skill: skill.to_string(),
        }
    }

    fn layout(&mut self) -> &mut view::Layout {
        &mut self.layout
    }

    fn handle_key_press(&mut self, key: KeyCode) -> HandleResult {
        HandleResult::Default
    }
}
