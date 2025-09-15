use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Rect},
    widgets::Clear,
};

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
        layout.add_el(Box::new(els::NameEl));
        Self { layout }
    }
}

impl view::Scene for SheetScene {
    fn layout(&self) -> &view::Layout {
        &self.layout
    }

    fn handle_key_press(&mut self, _key: KeyCode) -> HandleResult {
        HandleResult::Default
    }
}

pub struct SkillModal {
    layout: view::Layout,
    width: Constraint,
    height: Constraint,
    skill: String,
}

impl SkillModal {
    pub fn new(skill: &str, state: &State) -> Self {
        let editor = els::SkillProficiencyEditor::new(skill, state);
        let (width, height) = editor.dimensions().into();
        let mut layout = view::Layout::new();
        layout.add_el(Box::new(editor));
        Self {
            layout,
            width,
            height,
            skill: skill.to_string(),
        }
    }
}

impl Scene for SkillModal {
    fn layout(&self) -> &view::Layout {
        &self.layout
    }

    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        state: &State,
        selected: view::SelectedEl,
    ) -> Rect {
        let area = view::centre_in(frame.area(), self.width, self.height);
        frame.render_widget(Clear, area);
        self.layout.render(frame, area, state, selected);
        area
    }

    fn handle_key_press(&mut self, key: KeyCode) -> HandleResult {
        HandleResult::Default
    }
}
