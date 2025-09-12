use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEventKind},
    },
    prelude::*,
};

mod els;
// mod roll;
mod scenes;
mod stats;
mod view;

#[derive(Default)]
struct SheetState {
    name: String,
    stats: stats::Stats,
    skills: stats::Skills,
}

enum HandleResult {
    Close,
    Open(Box<dyn view::Scene>),
    Consume,
    Default,
}

struct SceneStackItem {
    scene: Box<dyn view::Scene>,
    position: view::SelectedEl,
}

impl SceneStackItem {
    fn new(scene: Box<dyn view::Scene>) -> Self {
        Self {
            scene,
            position: (0, 0),
        }
    }
}

struct App {
    state: SheetState,
    scene_stack: Vec<SceneStackItem>,
}

impl App {
    fn new() -> Self {
        Self {
            state: SheetState {
                name: "Character".to_string(),
                ..Default::default()
            },
            scene_stack: vec![SceneStackItem::new(Box::new(
                scenes::SheetScene::new(),
            ))],
        }
    }

    fn run(
        &mut self,
        term: &mut ratatui::DefaultTerminal,
    ) -> std::io::Result<()> {
        while !self.scene_stack.is_empty() {
            term.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        for item in self.scene_stack.iter_mut() {
            item.scene
                .layout()
                .render(frame, &self.state, item.position);
        }
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        // N.B. blocks until an event occurs.
        let event = ratatui::crossterm::event::read()?;
        let outcome = self.active_scene_mut().scene.handle(event.clone());
        if matches!(outcome, HandleResult::Default) {
            self.handle(event);
        } else {
            self.process_handle_result(outcome);
        }
        Ok(())
    }

    fn process_handle_result(&mut self, res: HandleResult) {
        match res {
            HandleResult::Close => {
                self.scene_stack.pop();
            }
            HandleResult::Open(scene) => {
                self.scene_stack.push(SceneStackItem::new(scene))
            }
            HandleResult::Consume | HandleResult::Default => {}
        }
    }

    fn handle(&mut self, event: Event) {
        if let Event::Key(evt) = event {
            if evt.kind == KeyEventKind::Press {
                self.handle_key_press(evt.code);
            }
        }
    }

    fn handle_key_press(&mut self, code: KeyCode) {
        if let Some(nav) = view::Navigation::from_key_code(code) {
            let old_position = self.active_scene().position;
            let new_position = self
                .scene_stack
                .last_mut()
                .unwrap()
                .scene
                .layout()
                .navigate(&self.state, old_position, nav);
            self.active_scene_mut().position = new_position;
            return;
        }

        match code {
            KeyCode::Char('q') => {
                self.scene_stack.pop();
            }
            KeyCode::Enter => {
                let pos = self.active_scene().position;
                let res = self
                    .scene_stack
                    .last_mut()
                    .unwrap()
                    .scene
                    .layout()
                    .select(&self.state, pos);
                self.process_handle_result(res);
            }
            _ => {}
        }
    }

    /// Return the top item from the scene stack.
    fn active_scene(&self) -> &SceneStackItem {
        // We exit when the scene stack is empty, so this unwrap is always
        // valid.
        self.scene_stack.last().unwrap()
    }

    /// Return a mutable reference to the top of the scene stack.
    fn active_scene_mut(&mut self) -> &mut SceneStackItem {
        self.scene_stack.last_mut().unwrap()
    }
}

fn main() -> std::io::Result<()> {
    let mut term = ratatui::init();
    crossterm::execute!(std::io::stdout(), crossterm::cursor::Hide).ok();
    let result = App::new().run(&mut term);
    ratatui::restore();
    crossterm::execute!(std::io::stdout(), crossterm::cursor::Show).ok();
    result
}
