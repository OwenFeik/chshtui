use std::collections::HashMap;

use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEventKind},
    },
    prelude::*,
    widgets::{Block, Clear, Paragraph},
};

mod els;
mod roll;
mod scenes;
mod stats;
mod view;

#[derive(Default)]
struct SheetState {
    name: String,
    stats: stats::Stats,
    skills: stats::Skills,
}

fn sheet_view() -> view::Scene {
    let mut v = view::Scene::new();
    stats::Stat::STATS
        .iter()
        .for_each(|s| v.add_el(Box::new(els::StatEl::new(*s))));
    v.add_group(Box::new(els::SkillsEl));
    v.add_column();
    v.add_el(Box::new(els::NameEl));
    v
}

#[derive(Eq, PartialEq)]
enum HandleResult {
    Consume,
    Default,
}

struct SceneStackItem {
    scene: Box<dyn view::Scene>,
    position: view::SelectedEl,
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
            scene_stack: vec![sheet_view()],
        }
    }

    fn run(
        &mut self,
        term: &mut ratatui::DefaultTerminal,
    ) -> std::io::Result<()> {
        while !self.should_close {
            term.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let Some(scene) = self.scenes.get_mut(&self.active_scene) else {
            return;
        };

        scene.draw(frame);

        if self.show_help {
            let entries = scene.help();
            let key_len =
                entries.iter().map(|e| e.key.len()).max().unwrap_or(0);
            let lines = entries
                .iter()
                .map(|e| format!("{0:<1$}    {2}", e.key, key_len, e.desc))
                .map(|s| Line::from(s))
                .collect::<Vec<Line>>();
            let paragraph = Paragraph::new(lines)
                .block(Block::bordered().title("help"))
                .style(Style::default().bg(Color::Black));

            let area = frame.area().inner(Margin::new(4, 2));
            frame.render_widget(Clear, area);
            frame.render_widget(paragraph, area);
        }
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        // N.B. blocks until an event occurs.
        let event = ratatui::crossterm::event::read()?;
        let outcome = self
            .scenes
            .get_mut(&self.active_scene)
            .map(|scene| scene.handle(event.clone()))
            .unwrap_or(HandleResult::Consume);

        match outcome {
            HandleResult::Consume => {}
            HandleResult::Default => {
                if let Event::Key(evt) = event {
                    if evt.kind == KeyEventKind::Press {
                        self.handle_key_press(evt.code);
                    }
                }
            }
        }
        Ok(())
    }
    fn handle(&mut self, event: Event) -> HandleResult {
        if let Event::Key(evt) = event {
            if evt.kind == KeyEventKind::Press {
                return match evt.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.position =
                            self.layout.up(self.position, &self.state);
                        HandleResult::Consume
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.position =
                            self.layout.down(self.position, &self.state);
                        HandleResult::Consume
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.position =
                            self.layout.left(self.position, &self.state);
                        HandleResult::Consume
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.position =
                            self.layout.right(self.position, &self.state);
                        HandleResult::Consume
                    }
                    _ => HandleResult::Default,
                };
            }
        }
        HandleResult::Default
    }

    fn handle_key_press(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('h') => self.show_help = true,
            KeyCode::Char('q') => {
                if self.show_help {
                    self.show_help = false;
                } else if self.active_scene == Scenes::Sheet {
                    self.should_close = true;
                } else {
                    self.active_scene = Scenes::Sheet;
                }
            }
            KeyCode::Char('r') => {
                self.active_scene = Scenes::Roll;
            }
            _ => (),
        }
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
