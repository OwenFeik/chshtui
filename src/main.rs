use crossterm::event::{Event, KeyCode, KeyEventKind};
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

#[derive(Default)]
struct App {
    should_exit: bool,
    stats: Stats,
}

impl App {
    fn run(
        &mut self,
        term: &mut ratatui::DefaultTerminal,
    ) -> std::io::Result<()> {
        while !self.should_exit {
            term.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Max(8), Constraint::Fill(1)])
            .split(frame.area());
        self.stats.render(layout[0], frame);
        frame.render_widget(self, layout[1]);
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        // N.B. blocks until an event occurs.
        match crossterm::event::read()? {
            Event::Key(evt)
                if evt.kind == KeyEventKind::Press
                    && evt.code == KeyCode::Char('q') =>
            {
                self.should_exit = true;
            }
            _ => (),
        }
        Ok(())
    }
}

impl ratatui::widgets::Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::bordered().title("Hello World").render(area, buf);
    }
}

fn main() -> std::io::Result<()> {
    let mut term = ratatui::init();
    let result = App::default().run(&mut term);
    ratatui::restore();
    result
}
