use std::collections::HashMap;

use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
    widgets::{Block, Clear, Paragraph},
};

mod roll;
mod stats;

#[derive(Default)]
struct SheetScene {
    stats: stats::Stats,
    skills: stats::Skills,
}

impl Scene for SheetScene {
    fn draw(&self, frame: &mut Frame) {
        let [stats_area, skills_area, rest] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Max(8),
                Constraint::Min(16),
                Constraint::Fill(1),
            ])
            .areas(frame.area());
        self.stats.render(stats_area, frame);
        frame.render_widget(self.skills.render(), skills_area);
        frame.render_widget(self, rest);
    }

    fn help(&self) -> &'static [HelpEntry] {
        &[
            HelpEntry {
                key: "h",
                desc: "Show help",
            },
            HelpEntry {
                key: "r",
                desc: "Roll dice",
            },
        ]
    }
}

impl Widget for &SheetScene {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::bordered().title("Sheet").render(area, buf);
    }
}

enum HandleResult {
    Consume,
    Default,
}

struct HelpEntry {
    key: &'static str,
    desc: &'static str,
}

trait Scene {
    /// Draw this scene into the frame buffer.
    fn draw(&self, frame: &mut Frame);

    /// Handle a terminal event that was performed.
    fn handle(&mut self, _event: Event) -> HandleResult {
        HandleResult::Default
    }

    /// Lines of help text to display.
    fn help(&self) -> &'static [HelpEntry];
}

#[derive(Hash, PartialEq, Eq)]
enum Scenes {
    Roll,
    Sheet,
}

struct App {
    scenes: HashMap<Scenes, Box<dyn Scene>>,
    active_scene: Scenes,
    show_help: bool,
    should_close: bool,
}

impl App {
    fn new() -> Self {
        let mut scenes: HashMap<Scenes, Box<dyn Scene>> = HashMap::new();
        scenes.insert(Scenes::Sheet, Box::new(SheetScene::default()));
        scenes.insert(Scenes::Roll, Box::new(roll::RollScene::new()));
        Self {
            scenes,
            active_scene: Scenes::Sheet,
            show_help: false,
            should_close: false,
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

    fn draw(&self, frame: &mut Frame) {
        let Some(scene) = self.scenes.get(&self.active_scene) else {
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
    let result = App::new().run(&mut term);
    ratatui::restore();
    result
}
