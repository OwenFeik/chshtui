use std::collections::HashMap;

use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
    widgets::{Block, Clear, Paragraph},
};

mod roll;
mod stats;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Location {
    Name,
    Stats,
    Skills,
    Sheet,
}

impl Location {
    fn up(self) -> Self {
        match self {
            Self::Name => self,
            Self::Stats => self,
            Self::Skills => Self::Name,
            Self::Sheet => self,
        }
    }

    fn down(self) -> Self {
        match self {
            Self::Name => Self::Skills,
            Self::Stats => self,
            Self::Skills => self,
            Self::Sheet => self,
        }
    }

    fn left(self) -> Self {
        match self {
            Self::Name => Self::Stats,
            Self::Stats => self,
            Self::Skills => Self::Stats,
            Self::Sheet => Self::Skills,
        }
    }

    fn right(self) -> Self {
        match self {
            Self::Name => Self::Sheet,
            Self::Stats => Self::Skills,
            Self::Skills => Self::Sheet,
            Self::Sheet => self,
        }
    }

    fn style<'a, T: Stylize<'a, T>>(&self, active: Self, w: T) -> T {
        if *self == active { w.bold() } else { w }
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::Name
    }
}

trait Element {
    fn render(&self, frame: &mut Frame, area: Rect, active: bool);

    fn location(&self) -> Location;

    fn handle(&mut self, key: KeyCode) -> HandleResult;
}

trait IndexedElement {
    fn render_indexed(&self, frame: &mut Frame, area: Rect, idx: Option<usize>);

    fn location(&self) -> Location;
}

struct Indexed<T: IndexedElement> {
    element: T,
    index: usize,
}

impl<T: IndexedElement> Element for Indexed<T> {
    fn render(&self, frame: &mut Frame, area: Rect, active: bool) {
        let idx = if active { Some(self.index) } else { None };
        self.element.render_indexed(frame, area, idx);
    }

    fn location(&self) -> Location {
        self.element.location()
    }

    fn handle(&mut self, key: KeyCode) -> HandleResult {
        match key {
            KeyCode::Up => {
                self.index += 1;
                HandleResult::Consume
            }
            KeyCode::Down => {
                self.index = self.index.saturating_sub(1);
                HandleResult::Consume
            }
            _ => HandleResult::Default,
        }
    }
}

impl<T> Default for Indexed<T>
where
    T: IndexedElement + Default,
{
    fn default() -> Self {
        Self {
            element: Default::default(),
            index: 0,
        }
    }
}

#[derive(Default)]
struct SheetScene {
    location: Location,
    name: String,
    stats: Indexed<stats::StatsElement>,
    skills: Indexed<stats::SkillsElement>,
}

impl SheetScene {
    fn render_element(&self, frame: &mut Frame, area: Rect, e: &dyn Element) {
        e.render(frame, area, self.location == e.location());
    }
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
        let [name_area, skills_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Max(3), Constraint::Fill(1)])
            .areas(skills_area);

        let name = Paragraph::new(self.name.as_str())
            .block(Block::bordered().title("Name"));
        frame.render_widget(
            self.location.style(Location::Name, name),
            name_area,
        );

        self.render_element(frame, stats_area, &self.stats);
        self.render_element(frame, skills_area, &self.skills);

        let sheet = self
            .location
            .style(Location::Sheet, Block::bordered().title("Sheet"));
        frame.render_widget(sheet, rest);
    }

    fn handle(&mut self, event: Event) -> HandleResult {
        if let Event::Key(evt) = event
            && evt.kind == KeyEventKind::Press
        {
            if self.location == Location::Stats
                && self.stats.handle(evt.code) == HandleResult::Consume
            {
                return HandleResult::Consume;
            }
            if self.location == Location::Skills
                && self.skills.handle(evt.code) == HandleResult::Consume
            {
                return HandleResult::Consume;
            }

            match evt.code {
                KeyCode::Up => {
                    self.location = self.location.up();
                    HandleResult::Consume
                }
                KeyCode::Down => {
                    self.location = self.location.down();
                    HandleResult::Consume
                }
                KeyCode::Left => {
                    self.location = self.location.left();
                    HandleResult::Consume
                }
                KeyCode::Right => {
                    self.location = self.location.right();
                    HandleResult::Consume
                }
                _ => HandleResult::Default,
            }
        } else {
            HandleResult::Default
        }
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

#[derive(Eq, PartialEq)]
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
