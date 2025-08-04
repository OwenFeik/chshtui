use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
    widgets::{Block, Paragraph},
};
use tui_input::{Input, backend::crossterm::EventHandler};

enum RollSuff {
    None,
    Advantage,
    Disadvantage,
    Keep(u32),
}

enum RollOp {
    Plus,
    Minus,
    Times,
    Divide,
}

impl RollOp {
    fn from(c: char) -> Option<RollOp> {
        match c {
            '+' => Some(RollOp::Plus),
            '-' => Some(RollOp::Minus),
            '*' | 'x' => Some(RollOp::Times),
            '/' => Some(RollOp::Divide),
            _ => None,
        }
    }

    fn format(&self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Times => "*",
            Self::Divide => "/",
        }
    }
}

struct RollMod {
    op: RollOp,
    amount: f64,
}

impl RollMod {
    fn apply(&self, to: f64) -> f64 {
        match self.op {
            RollOp::Plus => to + self.amount,
            RollOp::Minus => to - self.amount,
            RollOp::Times => to * self.amount,
            RollOp::Divide => to / self.amount,
        }
    }

    fn format(&self) -> String {
        format!("{} {}", self.op.format(), self.amount)
    }
}

struct Roll {
    quantity: u32,
    size: u32,
    suff: RollSuff,
    mods: Vec<RollMod>,
}

impl Roll {
    fn resolve(self) -> RollOutcome {
        let mut results = Vec::new();
        for _ in 0..self.quantity.max(1) {
            results.push(rand::random_range(1..=self.size));
        }

        let rolls_total = match self.suff {
            RollSuff::None => results.iter().copied().sum(),
            RollSuff::Advantage => results.iter().copied().max().unwrap_or(0),
            RollSuff::Disadvantage => {
                results.iter().copied().min().unwrap_or(0)
            }
            RollSuff::Keep(n) => {
                let mut sorted = results.clone();
                sorted.sort();
                sorted.reverse();
                let n = (n as usize).min(sorted.len());
                (&sorted[0..n]).iter().copied().sum()
            }
        };

        let mut value = rolls_total as f64;
        for modifier in &self.mods {
            value = modifier.apply(value);
        }

        RollOutcome {
            roll: self,
            results,
            value,
        }
    }
}

struct RollOutcome {
    roll: Roll,
    results: Vec<u32>,
    value: f64,
}

impl RollOutcome {
    fn format(&self) -> String {
        let mods = if self.roll.mods.is_empty() {
            String::new()
        } else {
            let mods = self
                .roll
                .mods
                .iter()
                .map(|m| m.format())
                .collect::<Vec<String>>()
                .join(" ");
            format!(" {mods}")
        };

        let roll = format!("{}d{}{}", self.roll.quantity, self.roll.size, mods);
        let nums = self
            .results
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        let val = if self.value.fract() == 0.0 {
            format!("{}", self.value)
        } else {
            format!("{:.2}", self.value)
        };

        format!("Roll: {roll}    Values: {nums}    Total: {val}")
    }
}

fn take_leading_int(text: &[char]) -> Option<(&[char], u32)> {
    let mut started = false;
    let mut val = 0;
    for (i, c) in text.iter().enumerate() {
        match c {
            _ if c.is_whitespace() && !started => (),
            v if v.is_digit(10) => {
                val *= 10;
                val += v.to_digit(10).unwrap();
                started = true;
            }
            _ if started => return Some((&text[i..], val)),
            _ => return None,
        }
    }

    if started { Some((&[], val)) } else { None }
}

fn take_leading_number(text: &[char]) -> Option<(&[char], f64)> {
    let mut num = String::new();
    let mut text = text;
    while let Some((rest, c)) = next_char(text) {
        match c {
            '.' => {
                if !num.contains('.') {
                    num.push('.');
                } else {
                    return num.parse().ok().map(|v| (text, v));
                }
            }
            _ if c.is_digit(10) => num.push(c),
            _ => return num.parse().ok().map(|v| (text, v)),
        }

        text = rest;
    }
    num.parse().ok().map(|v| (text, v))
}

fn next_char(text: &[char]) -> Option<(&[char], char)> {
    let mut stripped = text;
    while !stripped.is_empty() && stripped[0].is_whitespace() {
        stripped = &stripped[1..];
    }

    if !stripped.is_empty() {
        Some((&stripped[1..], stripped[0]))
    } else {
        None
    }
}

fn expect(expected: char, text: &[char]) -> Option<&[char]> {
    let (rest, actual) = next_char(text)?;
    if actual == expected { Some(rest) } else { None }
}

fn parse_roll_suff_mods(
    text: &[char],
) -> Option<(&[char], RollSuff, Vec<RollMod>)> {
    let mut suff = RollSuff::None;
    let mut mods = Vec::new();

    let (mut text, c) = next_char(text)?;
    match c {
        'a' => suff = RollSuff::Advantage,
        'd' => suff = RollSuff::Disadvantage,
        'k' => match take_leading_int(text) {
            Some((rest, num)) => {
                text = rest;
                suff = RollSuff::Keep(num);
            }
            None => suff = RollSuff::Keep(1),
        },
        c if RollOp::from(c).is_some() => {
            let op = RollOp::from(c).unwrap();
            let val;
            (text, val) = take_leading_number(text)?;
            mods.push(RollMod { op, amount: val });
        }
        _ => return None,
    }

    match parse_roll_suff_mods(text) {
        Some((text, suff, more_mods)) => {
            mods.extend(more_mods.into_iter());
            Some((text, suff, mods))
        }
        None => Some((text, suff, mods)),
    }
}

fn parse_roll(text: &[char]) -> Option<(Roll, &[char])> {
    let mut roll = Roll {
        quantity: 0,
        size: 0,
        suff: RollSuff::None,
        mods: Vec::new(),
    };

    let (text, quantity) = take_leading_int(text)?;
    roll.quantity = quantity;
    let text = expect('d', text)?;
    let (mut text, size) = take_leading_int(text)?;
    roll.size = size;
    if let Some((rest, suff, mods)) = parse_roll_suff_mods(text) {
        text = rest;
        roll.suff = suff;
        roll.mods = mods;
    }

    Some((roll, text))
}

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
    input: Input,
    rolls: Vec<String>,
}

impl RollScene {
    fn new() -> Self {
        Self {
            input: Input::new(String::new()),
            rolls: Vec::new(),
        }
    }
}

impl Scene for RollScene {
    fn draw(&self, frame: &mut Frame) {
        let [rolls_area, input_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Length(3)])
            .areas(frame.area());

        let lines: Vec<Line> =
            self.rolls.iter().map(|r| Line::from(r.as_str())).collect();
        let rolls = Paragraph::new(lines).block(
            Block::bordered()
                .title_alignment(Alignment::Center)
                .title("rolls"),
        );
        frame.render_widget(rolls, rolls_area);

        let input = Paragraph::new(self.input.value())
            .block(Block::bordered().title("input"));
        frame.render_widget(input, input_area);
    }

    fn handle(&mut self, event: Event) -> HandleResult {
        match event {
            Event::Key(evt) => match evt.code {
                KeyCode::Enter => {
                    let text = self.input.value_and_reset();
                    if let Some((roll, _)) = parse_roll(
                        text.chars().collect::<Vec<char>>().as_slice(),
                    ) {
                        self.rolls.push(roll.resolve().format());
                    } else {
                        self.rolls.push("Parse failure.".to_string());
                    }
                    HandleResult::Consume
                }
                KeyCode::Char('h') => HandleResult::Default,
                KeyCode::Char('l') => {
                    self.rolls.clear();
                    HandleResult::Consume
                }
                KeyCode::Char('q') => HandleResult::Default,
                KeyCode::Char('r') => HandleResult::Consume,
                _ => {
                    if self.input.handle_event(&event).is_some() {
                        HandleResult::Consume
                    } else {
                        HandleResult::Default
                    }
                }
            },
            _ => HandleResult::Default,
        }
    }

    fn help(&self) -> &'static [HelpEntry] {
        &[
            HelpEntry {
                key: "q",
                desc: "Close",
            },
            HelpEntry {
                key: "l",
                desc: "Clear rolls",
            },
            HelpEntry {
                key: "Enter",
                desc: "Submit roll",
            },
        ]
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
                .on_black();

            let area = frame.area().inner(Margin::new(4, 2));
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
