use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};

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

struct RollScene {
    input: String,
}

impl RollScene {
    fn new() -> Self {
        Self {
            input: String::new(),
        }
    }
}

impl Scene for RollScene {
    fn draw(&self, frame: &mut Frame) {
        let border = Block::bordered()
            .title_alignment(Alignment::Center)
            .title("Roll");
        let input = Paragraph::new(self.input.as_str())
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, frame.area());
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
}

impl Widget for &SheetScene {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::bordered().title("Hello World").render(area, buf);
    }
}

enum HandleResult {
    Consume,
    Default,
}

trait Scene {
    fn draw(&self, frame: &mut Frame);

    fn handle(&self, evt: Event) -> HandleResult {
        HandleResult::Default
    }
}

struct App {
    scenes: Vec<Box<dyn Scene>>,
}

impl App {
    fn new() -> Self {
        Self {
            scenes: vec![Box::new(SheetScene::default())],
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
        if let Some(scene) = self.scenes.last() {
            scene.draw(frame);
        }
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        // N.B. blocks until an event occurs.
        let event = crossterm::event::read()?;
        let outcome = self
            .scenes
            .last()
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
            KeyCode::Char('q') => {
                self.scenes.pop();
            }
            KeyCode::Char('r') => self.scenes.push(Box::new(RollScene::new())),
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
