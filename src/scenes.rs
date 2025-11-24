use ratatui::{crossterm::event::KeyCode, layout::Constraint};

use crate::{
    Handler, SheetState, editors,
    els::{self, BORDER},
    spells, stats, view,
};

pub struct SheetScene {
    layout: view::Layout<SheetState>,
}

impl SheetScene {
    pub fn new() -> Self {
        let mut layout = view::Layout::new();
        stats::Stat::STATS
            .iter()
            .for_each(|s| layout.add_el(els::StatEl::new(*s)));
        layout.add_group(els::SkillsEl);
        layout.add_column();
        layout.add_el(els::TextEl::new("Name", &|s| s.name.clone(), &|s| {
            Box::new(editors::StringEditorModal::new(
                "Name",
                s.name.clone(),
                Box::new(|value, state| state.name = value),
            ))
        }));
        layout.add_el(els::TextEl::new(
            "Level",
            &|s| format!("Level {}", s.level),
            &|s| {
                Box::new(editors::IntEditorModal::new(
                    "Level",
                    s.level,
                    Box::new(|level, state| state.level = level),
                ))
            },
        ));
        layout.add_el(els::SpellbookStatus);
        layout.add_group(els::Dice);
        layout.add_group(els::RollHistory::new(10));
        Self { layout }
    }
}

impl view::Scene<SheetState> for SheetScene {
    fn layout(&self) -> &view::Layout<SheetState> {
        &self.layout
    }
}

pub struct SpellbookScene {
    view: editors::EditorState<editors::SpellbookTablePos>,
    layout: view::Layout<SheetState>,
    search_input: editors::EditorState<String>,
}

impl SpellbookScene {
    pub fn new(state: &SheetState) -> Self {
        let (el, view) =
            editors::SpellbookTable::new(state.spellbook.query_all());
        let mut layout = view::Layout::new();
        layout.add_group(el);
        let (search_input, state) = editors::StringDisplay::new();
        layout.add_el(search_input);
        Self {
            view,
            layout,
            search_input: state,
        }
    }
}

impl view::Scene<SheetState> for SpellbookScene {
    fn layout(&self) -> &view::Layout<SheetState> {
        &self.layout
    }
}
