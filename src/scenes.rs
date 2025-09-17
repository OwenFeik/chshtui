use std::{cell::Cell, rc::Rc};

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

    fn handle_key_press(
        &mut self,
        _key: KeyCode,
        _state: &mut State,
    ) -> HandleResult {
        HandleResult::Default
    }
}

pub struct SkillModal {
    layout: view::Layout,
    width: Constraint,
    height: Constraint,
    skill: String,
    prof: Rc<Cell<stats::Proficiency>>,
}

impl SkillModal {
    pub fn new(skill: &str, state: &State) -> Self {
        let prof = Rc::new(Cell::new(
            state
                .skills
                .lookup(skill)
                .map(|s| s.proficiency)
                .unwrap_or(stats::Proficiency::Untrained),
        ));
        let editor = els::SkillProficiencyEditor::new(skill, prof.clone());
        let (width, height) = editor.dimensions().into();
        let mut layout = view::Layout::new();
        layout.add_el(Box::new(editor));
        Self {
            layout,
            width,
            height,
            skill: skill.to_string(),
            prof,
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

    fn handle_key_press(
        &mut self,
        key: KeyCode,
        state: &mut State,
    ) -> HandleResult {
        if key == KeyCode::Enter {
            if let Some(skill) = state.skills.lookup_mut(&self.skill) {
                skill.proficiency = self.prof.get();
            }
            return HandleResult::Close;
        }

        match view::Navigation::from_key_code(key) {
            Some(view::Navigation::Up) => {
                self.prof.set(self.prof.get().decrease());
                HandleResult::Consume
            }
            Some(view::Navigation::Down) => {
                self.prof.set(self.prof.get().increase());
                HandleResult::Consume
            }
            _ => HandleResult::Default,
        }
    }
}
