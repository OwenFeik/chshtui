use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEventKind, MouseEventKind},
    },
    prelude::*,
};

mod editors;
mod els;
mod fs;
mod roll;
mod scenes;
mod spells;
mod stats;
mod view;

const APP_NAME: &str = "chshtui";

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct SheetState {
    name: String,
    level: i64,
    stats: stats::Stats,
    skills: stats::Skills,

    #[serde(skip)]
    rolls: Vec<roll::RollOutcome>,
}

/// Handler for an input event.
type Handler = view::HandleResult<SheetState>;

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
    fn new(state: SheetState) -> Self {
        Self {
            state,
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
        if matches!(outcome, Handler::Default) {
            self.handle(event);
        } else {
            self.process_handle_result(outcome);
        }
        Ok(())
    }

    fn process_handle_result(&mut self, res: Handler) {
        match res {
            Handler::Close => {
                self.scene_stack.pop();
            }
            Handler::Open(scene) => {
                self.scene_stack.push(SceneStackItem::new(scene))
            }

            Handler::Replace(scene) => {
                self.scene_stack.pop();
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
    let save_file = match std::env::args().nth(1) {
        Some(path) => path.to_string(),
        None => "character.json".to_string(),
    };

    let state = match std::fs::File::open(&save_file) {
        Ok(file) => match serde_json::de::from_reader(file) {
            Ok(state) => state,
            Err(e) => {
                eprintln!("failed to parse save json {save_file}, error: {e}");
                std::process::exit(1);
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            SheetState::default()
        }
        Err(e) => {
            eprintln!("failed to read save data from {save_file}, error: {e}");
            std::process::exit(1);
        }
    };

    let mut app = App::new(state);

    let mut term = ratatui::init();
    crossterm::execute!(std::io::stdout(), crossterm::cursor::Hide).ok();
    let result = app.run(&mut term);
    ratatui::restore();
    crossterm::execute!(std::io::stdout(), crossterm::cursor::Show).ok();

    match serde_json::ser::to_string(&app.state) {
        Ok(json) => match std::fs::write(&save_file, json) {
            Ok(_) => println!("saved to {save_file}"),
            Err(e) => {
                eprintln!(
                    "failed to save character sheet to {save_file}, error: {e}"
                );
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("failed to format character sheet as json, error: {e}");
            std::process::exit(1);
        }
    }

    result
}
