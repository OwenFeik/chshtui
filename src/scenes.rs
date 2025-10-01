use crate::{SheetState, editors, els, stats, view};

pub struct SheetScene {
    layout: view::Layout<SheetState>,
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

impl view::Scene<SheetState> for SheetScene {
    fn layout(&self) -> &view::Layout<SheetState> {
        &self.layout
    }
}
