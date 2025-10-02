use ratatui::{
    Frame,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Rect},
    text::ToLine,
    widgets::{Row, Table},
};
use tui_input::backend::crossterm::EventHandler;

use crate::{
    els::{self, BORDER, State},
    roll, stats,
    view::{self, Dims, ElSimp, Handler, Scene},
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

struct CellDisplay<T: std::fmt::Display + Default + Clone> {
    shared_state: EditorState<T>,
    display_func: Box<dyn Fn(T) -> String>,
}

impl<T: std::fmt::Display + Default + Clone> CellDisplay<T> {
    fn new(
        shared_state: EditorState<T>,
        display_func: &'static dyn Fn(T) -> String,
    ) -> Self {
        Self {
            shared_state,
            display_func: Box::new(display_func),
        }
    }
    fn show(&self) -> String {
        (self.display_func)(self.shared_state.get())
    }
}

impl<T: std::fmt::Display + Default + Clone> ElSimp<State> for CellDisplay<T> {
    fn dimensions(&self) -> Dims {
        Dims::new(
            Constraint::Length(self.show().len() as u16),
            Constraint::Length(1),
        )
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &State,
        selected: bool,
    ) {
        frame.render_widget(
            els::style_selected(self.show().to_line().centered(), selected),
            area,
        );
    }
}

type EditorSubmitHandler<T> = Box<dyn FnMut(T, &mut State)>;

struct StringEditor {
    value: EditorState<String>,
}

impl ElSimp<State> for StringEditor {
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
        frame.render_widget(self.value.get().to_line(), area);
    }
}

pub struct StringEditorModal {
    layout: view::Layout<State>,
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
            value: value.clone(),
        };
        let mut layout = view::Layout::new();
        layout.add_el(el);
        let layout = layout.modal(
            title,
            Dims::new(Constraint::Min(24), Constraint::Length(1 + BORDER)),
            false,
        );

        Self {
            layout,
            apply_to_state: handler,
            value,
            input,
        }
    }
}

impl Scene<State> for StringEditorModal {
    fn layout(&self) -> &view::Layout<State> {
        &self.layout
    }

    fn handle(
        &mut self,
        event: Event,
        state: &mut State,
        _selected: view::ElPos,
    ) -> Handler {
        if let Event::Key(evt) = event
            && evt.kind == KeyEventKind::Press
        {
            let result = self.handle_key_press(evt.code, state);
            if !matches!(result, Handler::Default) {
                return result;
            }
        }

        match self.input.handle_event(&event) {
            Some(changes) => {
                if changes.value {
                    self.value.set(self.input.value().to_string());
                }
                Handler::Consume
            }
            None => Handler::Default,
        }
    }

    fn handle_key_press(&mut self, key: KeyCode, state: &mut State) -> Handler {
        match key {
            KeyCode::Enter => {
                (self.apply_to_state)(self.value.get(), state);
                Handler::Close
            }
            KeyCode::Esc => Handler::Close,
            _ => Handler::Default,
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

impl ElSimp<State> for SkillProficiencyEditor {
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
        let table =
            Table::default().rows(stats::Proficiency::ALL.iter().map(|p| {
                els::style_selected(Row::new([format!("{p:?}")]), *p == prof)
            }));

        frame.render_widget(table, area);
    }
}

pub struct SkillModal {
    layout: view::Layout<State>,
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
        layout.add_el(editor);
        Self {
            layout: layout.modal(skill, view::Dims::new(width, height), false),
            skill: skill.to_string(),
            eds,
        }
    }
}

impl Scene<State> for SkillModal {
    fn layout(&self) -> &view::Layout<State> {
        &self.layout
    }

    fn handle_key_press(&mut self, key: KeyCode, state: &mut State) -> Handler {
        if key == KeyCode::Enter {
            if let Some(skill) = state.skills.lookup_mut(&self.skill) {
                skill.proficiency = self.eds.get();
            }
            return Handler::Close;
        }

        match view::Navigation::from_key_code(key) {
            Some(view::Navigation::Up) => {
                self.eds.update(|p| p.decrease());
                Handler::Consume
            }
            Some(view::Navigation::Down) => {
                self.eds.update(|p| p.increase());
                Handler::Consume
            }
            _ => Handler::Default,
        }
    }
}

struct IntEditor {
    state: EditorState<i64>,
}

impl IntEditor {
    fn new(initial_value: i64) -> (EditorState<i64>, Self) {
        let state = EditorState::new(initial_value);
        (state.clone(), Self { state })
    }
}

impl ElSimp<State> for IntEditor {
    fn dimensions(&self) -> Dims {
        Dims::new(
            Constraint::Length(self.state.get().to_string().len() as u16 + 4),
            Constraint::Length(1),
        )
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &State,
        _selected: bool,
    ) {
        let value = self.state.get();
        let score = format!("< {value} >");
        let line = score.to_line().centered();
        frame.render_widget(line, area);
    }
}

pub struct IntEditorModal {
    layout: view::Layout<State>,
    eds: EditorState<i64>,
    handler: EditorSubmitHandler<i64>,
}

impl IntEditorModal {
    pub fn new(
        title: &str,
        initial_value: i64,
        handler: EditorSubmitHandler<i64>,
    ) -> Self {
        let (eds, editor) = IntEditor::new(initial_value);
        let dimensions = Dims::new(
            Constraint::Length(2 + 2 + 2 + BORDER),
            Constraint::Length(1 + BORDER),
        );
        let mut layout = view::Layout::new();
        layout.add_el(editor);
        Self {
            layout: layout.modal(title, dimensions, false),
            eds,
            handler,
        }
    }
}

impl Scene<State> for IntEditorModal {
    fn layout(&self) -> &view::Layout<State> {
        &self.layout
    }

    fn handle_key_press(&mut self, key: KeyCode, state: &mut State) -> Handler {
        if key == KeyCode::Enter {
            (self.handler)(self.eds.get(), state);
            return Handler::Close;
        }

        match view::Navigation::from_key_code(key) {
            Some(view::Navigation::Left) => {
                self.eds.update(|s| s - 1);
                Handler::Consume
            }
            Some(view::Navigation::Right) => {
                self.eds.update(|s| s + 1);
                Handler::Consume
            }
            _ => Handler::Default,
        }
    }
}

pub fn stat_modal(stat: stats::Stat, state: &State) -> Box<dyn Scene<State>> {
    let score = state.stats.score(stat);
    let mut modal = IntEditorModal::new(
        "",
        score,
        Box::new(move |score, state| state.stats.set_score(stat, score)),
    );
    let modifier = CellDisplay::new(modal.eds.clone(), &|score| {
        els::format_modifier(stats::Stat::modifier(score))
    });
    modal.layout.add_el(modifier);
    let dimensions = Dims::new(
        Constraint::Length(2 + 2 + 2 + BORDER),
        Constraint::Length(2 + BORDER),
    );
    modal.layout = modal.layout.modal(&stat.short(), dimensions, false);
    Box::new(modal)
}

pub struct RollModal {
    outcome: roll::RollOutcome,
    layout: view::Layout<State>,
}

impl RollModal {
    pub fn new(r: roll::Roll) -> Self {
        let outcome = r.resolve();
        let mut layout = view::Layout::new();
        let element = els::RollDisplay::new(&outcome);
        let width = if let Constraint::Length(w) = element.dimensions().width()
        {
            w + BORDER
        } else {
            16
        };
        let width = Constraint::Length(width);
        let height = Constraint::Length(2 + BORDER);
        let dimensions = Dims::new(width, height);
        layout.add_el(element);
        Self {
            layout: layout.modal("Roll", dimensions, false),
            outcome,
        }
    }
}

impl Scene<State> for RollModal {
    fn layout(&self) -> &view::Layout<State> {
        &self.layout
    }

    fn handle_key_press(
        &mut self,
        _key: KeyCode,
        _state: &mut State,
    ) -> Handler {
        Handler::Default
    }
}
