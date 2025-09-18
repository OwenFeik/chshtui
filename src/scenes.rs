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

pub struct SkillModal {
    layout: view::Layout,
    skill: String,
    eds: els::EditorState<stats::Proficiency>,
}

impl SkillModal {
    pub fn new(skill: &str, state: &State) -> Self {
        let prof = state
            .skills
            .lookup(skill)
            .map(|s| s.proficiency)
            .unwrap_or(stats::Proficiency::Untrained);
        let (eds, editor) = els::SkillProficiencyEditor::new(skill, prof);
        let (width, height) = editor.dimensions().into();
        let mut layout = view::Layout::new();
        layout.add_el(Box::new(editor));
        Self {
            layout: layout.modal(view::Dims::new(width, height)),
            skill: skill.to_string(),
            eds,
        }
    }
}

impl Scene for SkillModal {
    fn layout(&self) -> &view::Layout {
        &self.layout
    }

    fn handle_key_press(
        &mut self,
        key: KeyCode,
        state: &mut State,
    ) -> HandleResult {
        if key == KeyCode::Enter {
            if let Some(skill) = state.skills.lookup_mut(&self.skill) {
                skill.proficiency = self.eds.get();
            }
            return HandleResult::Close;
        }

        match view::Navigation::from_key_code(key) {
            Some(view::Navigation::Up) => {
                self.eds.update(|p| p.decrease());
                HandleResult::Consume
            }
            Some(view::Navigation::Down) => {
                self.eds.update(|p| p.increase());
                HandleResult::Consume
            }
            _ => HandleResult::Default,
        }
    }
}

pub struct StatModal {
    layout: view::Layout,
    stat: stats::Stat,
    eds: els::EditorState<i64>,
}

impl StatModal {
    pub fn new(stat: stats::Stat, state: &State) -> Self {
        let score = state.stats.score(stat);
        let (eds, editor) = els::StatEditor::new(stat, score);
        let dimensions = editor.dimensions();
        let mut layout = view::Layout::new();
        layout.add_el(Box::new(editor));
        Self {
            layout: layout.modal(dimensions),
            stat,
            eds,
        }
    }
}

impl Scene for StatModal {
    fn layout(&self) -> &view::Layout {
        &self.layout
    }

    fn handle_key_press(
        &mut self,
        key: KeyCode,
        state: &mut State,
    ) -> HandleResult {
        if key == KeyCode::Enter {
            state.stats.set_score(self.stat, self.eds.get());
            return HandleResult::Close;
        }

        match view::Navigation::from_key_code(key) {
            Some(view::Navigation::Left) => {
                self.eds.update(|s| s - 1);
                HandleResult::Consume
            }
            Some(view::Navigation::Right) => {
                self.eds.update(|s| s + 1);
                HandleResult::Consume
            }
            _ => HandleResult::Default,
        }
    }
}
