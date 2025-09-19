use ratatui::{
    Frame,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{self, Alignment, Constraint, Margin, Rect},
    text::ToLine,
    widgets::{Block, Paragraph, Row, Table},
};
use tui_input::backend::crossterm::EventHandler;

use crate::{
    HandleResult,
    els::{self, BORDER},
    stats,
    view::{self, Dims, ElSimp, Scene, State},
};

#[derive(Clone)]
struct EditorState<T: Clone + Default> {
    shared_state: std::rc::Rc<std::cell::Cell<T>>,
}

impl<T: Clone + Default> EditorState<T> {
    fn new(initial_value: T) -> Self {
        Self {
            shared_state: std::rc::Rc::new(std::cell::Cell::new(initial_value)),
        }
    }

    fn get(&self) -> T {
        let value = self.shared_state.take();
        self.set(value.clone());
        value
    }

    fn set(&self, value: T) {
        self.shared_state.set(value);
    }

    fn update(&self, effect: impl FnOnce(T) -> T) {
        let value = self.shared_state.take();
        self.shared_state.set(effect(value));
    }
}

type EditorSubmitHandler<T> = Box<dyn FnMut(T, &mut State)>;

struct StringEditor {
    title: String,
    value: EditorState<String>,
}

impl ElSimp for StringEditor {
    fn dimensions(&self) -> Dims {
        Dims::new(Constraint::Min(16), Constraint::Length(3))
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &State,
        _selected: bool,
    ) {
        let widget = Paragraph::new(self.value.get())
            .block(Block::bordered().title(self.title.as_str()));
        frame.render_widget(widget, area);
    }
}

pub struct StringEditorModal {
    layout: view::Layout,
    apply_to_state: EditorSubmitHandler<String>,
    value: EditorState<String>,
    input: tui_input::Input,
}

impl StringEditorModal {
    pub fn new(
        title: &str,
        initial_value: String,
        handler: EditorSubmitHandler<String>,
    ) -> Self {
        let input = tui_input::Input::new(initial_value.clone());
        let value = EditorState::new(initial_value);
        let el = StringEditor {
            title: title.to_string(),
            value: value.clone(),
        };
        let mut layout = view::Layout::new();
        layout.add_el(Box::new(el));
        let layout = layout.modal(Dims::new(
            Constraint::Min(24),
            Constraint::Length(1 + BORDER),
        ));

        Self {
            layout,
            apply_to_state: handler,
            value,
            input,
        }
    }
}

impl Scene for StringEditorModal {
    fn layout(&self) -> &view::Layout {
        &self.layout
    }

    fn handle(&mut self, event: Event, state: &mut State) -> HandleResult {
        if let Event::Key(evt) = event
            && evt.kind == KeyEventKind::Press
        {
            let result = self.handle_key_press(evt.code, state);
            if !matches!(result, HandleResult::Default) {
                return result;
            }
        }

        match self.input.handle_event(&event) {
            Some(changes) => {
                if changes.value {
                    self.value.set(self.input.value().to_string());
                }
                HandleResult::Consume
            }
            None => HandleResult::Default,
        }
    }

    fn handle_key_press(
        &mut self,
        key: KeyCode,
        state: &mut State,
    ) -> HandleResult {
        match key {
            KeyCode::Enter => {
                (self.apply_to_state)(self.value.get(), state);
                HandleResult::Close
            }
            KeyCode::Esc => HandleResult::Close,
            _ => HandleResult::Default,
        }
    }
}

struct SkillProficiencyEditor {
    skill: String,
    state: EditorState<stats::Proficiency>,
}

impl SkillProficiencyEditor {
    fn new(
        skill: &str,
        prof: stats::Proficiency,
    ) -> (EditorState<stats::Proficiency>, Self) {
        let state = EditorState::new(prof);
        (
            state.clone(),
            Self {
                skill: skill.to_string(),
                state,
            },
        )
    }
}

impl ElSimp for SkillProficiencyEditor {
    fn dimensions(&self) -> Dims {
        Dims::new(
            Constraint::Min(self.skill.len() as u16 + BORDER),
            Constraint::Length(stats::Proficiency::ALL.len() as u16 + BORDER),
        )
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &State,
        _selected: bool,
    ) {
        let prof = self.state.get();
        let table = Table::default()
            .rows(stats::Proficiency::ALL.iter().map(|p| {
                els::style_selected(Row::new([format!("{p:?}")]), *p == prof)
            }))
            .block(Block::bordered().title(self.skill.as_str()));

        frame.render_widget(table, area);
    }
}

pub struct SkillModal {
    layout: view::Layout,
    skill: String,
    eds: EditorState<stats::Proficiency>,
}

impl SkillModal {
    pub fn new(skill: &str, state: &State) -> Self {
        let prof = state
            .skills
            .lookup(skill)
            .map(|s| s.proficiency)
            .unwrap_or(stats::Proficiency::Untrained);
        let (eds, editor) = SkillProficiencyEditor::new(skill, prof);
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

struct StatEditor {
    stat: stats::Stat,
    state: EditorState<i64>,
}

impl StatEditor {
    fn new(stat: stats::Stat, initial_value: i64) -> (EditorState<i64>, Self) {
        let state = EditorState::new(initial_value);
        (state.clone(), Self { stat, state })
    }
}

impl ElSimp for StatEditor {
    fn dimensions(&self) -> Dims {
        Dims::new(
            Constraint::Length(6 + BORDER),
            Constraint::Length(2 + BORDER),
        )
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &State,
        _selected: bool,
    ) {
        let score = self.state.get();
        let modifier = els::format_modifier(stats::Stat::modifier(score));
        let score = format!("< {score} >");
        let widget = Paragraph::new(vec![score.to_line(), modifier.to_line()])
            .centered()
            .block(
                Block::bordered()
                    .title(self.stat.short())
                    .title_alignment(Alignment::Center),
            );

        frame.render_widget(widget, area);
    }
}

pub struct StatModal {
    layout: view::Layout,
    stat: stats::Stat,
    eds: EditorState<i64>,
}

impl StatModal {
    pub fn new(stat: stats::Stat, state: &State) -> Self {
        let score = state.stats.score(stat);
        let (eds, editor) = StatEditor::new(stat, score);
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
