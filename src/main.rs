use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEventKind, MouseEventKind},
    },
    prelude::*,
};

mod editors;
mod els;
mod roll;
mod scenes;
mod stats;
mod view;

#[derive(Default)]
struct SheetState {
    name: String,
    level: i64,
    stats: stats::Stats,
    skills: stats::Skills,
    rolls: Vec<roll::RollOutcome>,
}

struct SceneStackItem {
    scene: Box<dyn view::Scene<SheetState>>,
    dimensions: Rect,
    position: view::ElPos,
}

impl SceneStackItem {
    fn new(scene: Box<dyn view::Scene<SheetState>>) -> Self {
        Self {
            scene,
            dimensions: Default::default(),
            position: view::ElPos::default(),
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
            item.dimensions =
                item.scene
                    .layout()
                    .render(frame, &self.state, item.position);
        }
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        // N.B. blocks until an event occurs.
        let event = ratatui::crossterm::event::read()?;
        let active = self.scene_stack.last_mut().unwrap();
        let outcome = active.scene.handle(
            event.clone(),
            &mut self.state,
            active.position,
        );
        if matches!(outcome, view::Handler::Default) {
            self.handle(event);
        } else {
            self.process_handle_result(outcome);
        }
        Ok(())
    }

    fn process_handle_result(&mut self, res: view::Handler) {
        use view::Handler;
        match res {
            Handler::Close => {
                self.scene_stack.pop();
            }
            Handler::Open(scene) => {
                self.scene_stack.push(SceneStackItem::new(scene))
            }
            Handler::Consume | Handler::Default => {}
        }
    }

    fn handle(&mut self, event: Event) {
        match event {
            Event::Key(evt) => {
                if evt.kind == KeyEventKind::Press {
                    self.handle_key_press(evt.code);
                }
            }
            Event::Mouse(evt) => {
                if let MouseEventKind::Down(_) = evt.kind {
                    let active = self.active_scene();
                    let area = active.dimensions;
                    let position = active.scene.layout().element_at_coordinate(
                        area,
                        &self.state,
                        evt.column,
                        evt.row,
                    );
                    self.active_scene_mut().position = position;
                }
            }
            _ => (),
        }
    }

    fn handle_key_press(&mut self, code: KeyCode) {
        if let Some(nav) = view::Navigation::from_key_code(code) {
            let active = self.active_scene();
            let new_position = self.active_scene().scene.layout().navigate(
                active.dimensions,
                &self.state,
                active.position,
                nav,
            );
            self.active_scene_mut().position = new_position;
            return;
        }

        match code {
            KeyCode::Char('q') => {
                self.scene_stack.pop();
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
