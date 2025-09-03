use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

pub type State = crate::SheetState;

pub struct Dims {
    x: Constraint,
    y: Constraint,
}

impl Dims {
    pub fn new(width: Constraint, height: Constraint) -> Self {
        Dims {
            x: width,
            y: height,
        }
    }
}

pub trait ElSimp {
    fn dimensions(&self) -> Dims;

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &State,
        selected: bool,
    );
}

pub trait ElGroup {
    fn dimensions(&self, state: &State) -> Dims;

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &State,
        selected: Option<usize>,
    );

    fn child_count(&self, state: &State) -> usize;

    fn child_y(&self, area: Rect, state: &State, selected: usize) -> u16;
}

enum El {
    Simple(Box<dyn ElSimp>),
    Group(Box<dyn ElGroup>),
}

impl El {
    fn dimensions(&self, state: &State) -> Dims {
        match self {
            Self::Simple(el) => el.dimensions(),
            Self::Group(el) => el.dimensions(state),
        }
    }

    fn children(&self, state: &State) -> usize {
        match self {
            Self::Simple(_) => 1,
            Self::Group(group) => group.child_count(state),
        }
    }
}

fn compare_constraints(a: &Constraint, b: &Constraint) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (Constraint::Min(a), Constraint::Min(b)) => a.cmp(&b),
        (Constraint::Max(a), Constraint::Max(b)) => a.cmp(&b),
        (Constraint::Length(a), Constraint::Length(b)) => a.cmp(&b),
        (Constraint::Percentage(a), Constraint::Percentage(b)) => a.cmp(&b),
        (Constraint::Ratio(a1, a2), Constraint::Ratio(b1, b2)) => {
            (*a1 as f32 / *a2 as f32).total_cmp(&(*b1 as f32 / *b2 as f32))
        }
        (Constraint::Fill(a), Constraint::Fill(b)) => a.cmp(&b),
        (Constraint::Max(_), _) => Ordering::Greater,
        (_, Constraint::Max(_)) => Ordering::Less,
        (Constraint::Length(_), _) => Ordering::Greater,
        (_, Constraint::Length(_)) => Ordering::Less,
        (Constraint::Min(_), _) => Ordering::Greater,
        (_, Constraint::Min(_)) => Ordering::Less,
        (Constraint::Percentage(_), _) => Ordering::Greater,
        (_, Constraint::Percentage(_)) => Ordering::Less,
        (Constraint::Ratio(..), _) => Ordering::Greater,
        (_, Constraint::Ratio(..)) => Ordering::Less,
    }
}

struct Column {
    elements: Vec<El>,
}

impl Column {
    fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    fn width(&self, state: &State) -> Constraint {
        self.elements
            .iter()
            .map(|e| e.dimensions(state).x)
            .max_by(compare_constraints)
            .unwrap_or(Constraint::Fill(0))
    }

    fn layout(&self, state: &State) -> Layout {
        Layout::new(
            Direction::Vertical,
            self.elements.iter().map(|e| e.dimensions(state).y),
        )
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &State,
        selected: Option<usize>,
    ) {
        let mut selected = selected.unwrap_or(usize::MAX);
        let areas = self.layout(state).split(area);
        for (&area, element) in areas.iter().zip(self.elements.iter()) {
            let child_count = element.children(state);
            match element {
                El::Simple(el) => el.render(frame, area, state, selected == 0),
                El::Group(group) => {
                    let child_index = if selected < child_count {
                        Some(selected)
                    } else {
                        None
                    };
                    group.render(frame, area, state, child_index);
                }
            }
            selected = selected.saturating_sub(child_count);
        }
    }
}

pub struct View {
    columns: Vec<Column>,
}

impl View {
    pub fn new() -> Self {
        Self {
            columns: vec![Column::new()],
        }
    }

    fn layout(&self, state: &State) -> Layout {
        Layout::new(
            Direction::Horizontal,
            self.columns.iter().map(|e| e.width(state)),
        )
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        state: &State,
        (x, y): (usize, usize),
    ) {
        let areas = self.layout(state).split(frame.area());
        for (i, (&area, column)) in
            areas.iter().zip(self.columns.iter()).enumerate()
        {
            let selected_index = if x == i { Some(y) } else { None };
            column.render(frame, area, state, selected_index);
        }
    }

    pub fn el(&mut self, el: Box<dyn ElSimp>) {
        if let Some(column) = self.columns.last_mut() {
            column.elements.push(El::Simple(el));
        }
    }

    pub fn group(&mut self, group: Box<dyn ElGroup>) {
        if let Some(column) = self.columns.last_mut() {
            column.elements.push(El::Group(group));
        }
    }

    pub fn column(&mut self) {
        self.columns.push(Column {
            elements: Vec::new(),
        });
    }
}
