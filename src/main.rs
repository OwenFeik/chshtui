use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
    widgets::{Block, Clear, Paragraph},
};

mod roll;

struct Stat {
    name: String,
    value: i8,
}

impl Stat {
    fn short(&self) -> String {
        if self.name.len() < 3 {
            self.name.clone()
        } else {
            self.name[0..3].to_uppercase()
        }
    }

    fn modifier(&self) -> i64 {
        ((self.value - 10) / 2) as i64
    }

    fn mod_string(&self) -> String {
        let modifier = self.modifier();
        if modifier < 0 {
            modifier.to_string()
        } else {
            format!("+{modifier}")
        }
    }
}

impl Widget for &Stat {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(self.short());
        Paragraph::new(vec![
            Line::from(self.value.to_string()),
            Line::from(self.mod_string()),
        ])
        .centered()
        .block(block)
        .render(area, buf);
    }
}

struct Stats {
    stats: Vec<Stat>,
}

impl Stats {
    fn render(&self, area: Rect, frame: &mut Frame) {
        let constraints: Vec<Constraint> =
            std::iter::repeat(Constraint::Ratio(1, self.stats.len() as u32))
                .take(self.stats.len())
                .collect();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);
        for i in 0..self.stats.len() {
            frame.render_widget(&self.stats[i], layout[i]);
        }
    }
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            stats: vec![
                Stat {
                    name: "Dexterity".into(),
                    value: 10,
                },
                Stat {
                    name: "Strength".into(),
                    value: 10,
                },
                Stat {
                    name: "Constitution".into(),
                    value: 10,
                },
                Stat {
                    name: "Charisma".into(),
                    value: 10,
                },
                Stat {
                    name: "Wisdom".into(),
                    value: 10,
                },
                Stat {
                    name: "Intelligence".into(),
                    value: 10,
                },
            ],
        }
    }
}

#[derive(Default)]
struct SheetScene {
    stats: Stats,
}
impl Scene for SheetScene {
    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Max(8), Constraint::Fill(1)])
            .split(frame.area());
        self.stats.render(layout[0], frame);
        frame.render_widget(self, layout[1]);
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
        Block::bordered().title("sheet").render(area, buf);
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

struct App {
    scenes: Vec<Box<dyn Scene>>,
    show_help: bool,
}

impl App {
    fn new() -> Self {
        Self {
            scenes: vec![Box::new(SheetScene::default())],
            show_help: false,
        }
    }

    fn run(
        &mut self,
        term: &mut ratatui::DefaultTerminal,
    ) -> std::io::Result<()> {
        while !self.scenes.is_empty() {
            term.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let Some(scene) = self.scenes.last() else {
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
            .last_mut()
            .map(|scene| scene.handle(event.clone()))
            .unwrap_or(HandleResult::Consume);

        match outcome {
            HandleResult::Consume => {}
            HandleResult::Default => {
                if let Event::Key(evt) = event
                    && evt.kind == KeyEventKind::Press
                {
                    self.handle_key_press(evt.code);
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
                } else {
                    self.scenes.pop();
                }
            }
            KeyCode::Char('r') => {
                self.scenes.push(Box::new(roll::RollScene::new()))
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
