use ratatui::{
    Frame,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Position, Rect},
};

use crate::HandleResult;

/// Application state type, reference provided to elements during rendering.
pub type State = crate::SheetState;

/// A scene is a full-screen (potentially floating) view of the application.
/// A scene is expected to track non-application global state internally and
/// update it appropriately based on user into.
pub trait Scene {
    /// Returns a mutable reference to the scene's layout for rendering to the
    /// screen.
    fn layout(&mut self) -> &mut Layout;

    /// Handle user input entered while the scene is active. Should return
    /// [HandleResult::Consume] if the input was used to update state, or
    /// [HandleResult::Default] if the parent context should handle the event.
    ///
    /// The default implementation ignores all events except keypresses and
    /// delegates keypresses to [Scene::handle_key_press].
    fn handle(&mut self, event: Event) -> HandleResult {
        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                return self.handle_key_press(key_event.code);
            }
        }
        HandleResult::Default
    }

    /// Handle a key press while the scene was active. If the keypress is
    /// used by the scene, [HandleResult::Consume] should be returned.
    /// Otherwise [HandleResult::Default] can be returned to delegate handling
    /// to the parent context.
    ///
    /// Navigation with arrows/hjkl and exit with q are handled by the global
    /// context automatically.
    fn handle_key_press(&mut self, key: KeyCode) -> HandleResult;
}

/// Element dimension constraints.
pub struct Dims {
    x: Constraint,
    y: Constraint,
}

impl Dims {
    /// Create new constraints on width and height.
    pub fn new(width: Constraint, height: Constraint) -> Self {
        Dims {
            x: width,
            y: height,
        }
    }
}

/// Trait for simple elements, single elements selected as a whole.
pub trait ElSimp {
    /// Return dimension constraints for this element.
    fn dimensions(&self) -> Dims;

    /// Render this element to the frame in the provided area, based on the
    /// current state. If selected is indicated the element should be styled
    /// appropriately.
    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &State,
        selected: bool,
    );
}

/// Trait for grouped elements, which are rendered as a single element but
/// selected individually. For example a table with individually selectable
/// rows.
pub trait ElGroup {
    /// Return dimensions for the whole element group.
    fn dimensions(&self, state: &State) -> Dims;

    /// Render this group of elements into the provided area, based on the
    /// current state. If any element in the group is selected, its index
    /// within the group will be provided, otherwise selected will be None.
    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &State,
        selected: Option<usize>,
    );

    /// Return the number of child elements in this group, for selection
    /// handling.
    fn child_count(&self, state: &State) -> usize;

    /// Calculate and return the y position of the top of an element in this
    /// group. Area is the area of the whole group. The y position should
    /// returned should be the top of the child element at index selected.
    fn child_y(&self, area: Rect, state: &State, selected: usize) -> u16;

    /// Get the index of the child element within this element that is offset
    /// by y_offset lines from the top of this element. Return the index of
    /// that child element within this group.
    fn child_at_y(&self, state: &State, y_offset: u16) -> usize;
}

/// Elements which can appear in view columns. Each element is either a simple
/// single element or a group of elements rendered together.
enum El {
    Simple(Box<dyn ElSimp>),
    Group(Box<dyn ElGroup>),
}

impl El {
    /// Return the dimension constraints for this element.
    fn dimensions(&self, state: &State) -> Dims {
        match self {
            Self::Simple(el) => el.dimensions(),
            Self::Group(el) => el.dimensions(state),
        }
    }

    /// Return the number of child elements for this element. For simple
    /// elements this is always just 1. For groups this is the number of
    /// selectable child elements.
    fn child_count(&self, state: &State) -> usize {
        match self {
            Self::Simple(_) => 1,
            Self::Group(group) => group.child_count(state),
        }
    }
}

/// Compare two ratatui [Constraint]s, ordering such that more constraining
/// constraints are placed first. This is used to prioritise constraints when
/// laying out columns.
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

/// A column in the view contains any number of elements rendered top to
/// bottom.
struct Column {
    elements: Vec<El>,
}

impl Column {
    /// Create a new empty column.
    fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Return a constraint for the width of this column in the overall view.
    /// This will be the most constraining constraint of any child element in
    /// the column.
    fn width(&self, state: &State) -> Constraint {
        self.elements
            .iter()
            .map(|e| e.dimensions(state).x)
            .max_by(compare_constraints)
            .unwrap_or(Constraint::Fill(0))
    }

    /// Returns a ratatui layout for this column to lay out child elements for
    /// rendering.
    fn layout(&self, state: &State) -> ratatui::layout::Layout {
        ratatui::layout::Layout::new(
            Direction::Vertical,
            self.elements.iter().map(|e| e.dimensions(state).y),
        )
    }

    /// Iterate across pairs of element and area in layed-out column for
    /// rendering or position calculation.
    fn iter_layout(
        &self,
        state: &State,
        area: Rect,
    ) -> impl Iterator<Item = (&El, Rect)> {
        let areas = self.layout(state).split(area).to_vec();
        self.elements.iter().zip(areas.into_iter())
    }

    /// Render the column into the provided area based on the current state.
    /// This will render all elements in the column, top to bottom, with the
    /// selected element appropriately styled.
    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &State,
        selected: Option<usize>,
    ) {
        let mut selected = selected.unwrap_or(usize::MAX);
        for (element, area) in self.iter_layout(state, area) {
            let child_count = element.child_count(state);
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
            selected = selected.wrapping_sub(child_count);
        }
    }

    /// Count the number of selectable elements in this column.
    fn child_count(&self, state: &State) -> usize {
        self.elements.iter().map(|e| e.child_count(state)).sum()
    }

    /// Calculate the y-value of the top of an element in this column when
    /// rendered into an area of the provided size based on element selection
    /// index.
    fn child_y(&self, area: Rect, state: &State, selected: usize) -> u16 {
        let mut selected = selected;
        for (element, area) in self.iter_layout(state, area) {
            let child_count = element.child_count(state);
            if selected < child_count {
                return match element {
                    El::Simple(_) => area.y,
                    El::Group(group) => group.child_y(area, state, selected),
                };
            }
            selected = selected.saturating_sub(child_count);
        }
        0
    }

    /// Calculate the selection index in this column for a given y coordinate.
    /// This will calculate the layouts for elements in the column and
    /// determine which element the provided y position falls into.
    fn child_at_y(&self, area: Rect, state: &State, y: u16) -> usize {
        let mut index = 0;
        for (element, area) in self.iter_layout(state, area) {
            if area.contains(Position::new(area.x, y)) {
                return match element {
                    El::Simple(_) => index,
                    El::Group(group) => {
                        index
                            + group.child_at_y(state, y.saturating_sub(area.y))
                    }
                };
            }
            index += element.child_count(state);
        }
        0
    }
}

/// Selection coordinate into the view. Pair of (col, row) where col is the
/// index of the column of the selected element and row is the selection index
/// within the column of the element. Not that this does not just resolve to
/// columns[col].elements[row] because elements in a column may have multiple
/// selected children.
pub type SelectedEl = (usize, usize);

/// A movement around a layout.
pub enum Navigation {
    Up,
    Down,
    Left,
    Right,
}

impl Navigation {
    /// Return the navigation the provided keycode maps to, if any.
    pub fn from_key_code(code: KeyCode) -> Option<Navigation> {
        match code {
            KeyCode::Up | KeyCode::Char('k') => Some(Self::Up),
            KeyCode::Down | KeyCode::Char('j') => Some(Self::Down),
            KeyCode::Left | KeyCode::Char('h') => Some(Self::Left),
            KeyCode::Right | KeyCode::Char('l') => Some(Self::Right),
            _ => None,
        }
    }
}

/// View of the application state. Handles rendering the ratatui TUI based on
/// the current state and the provided elements.
pub struct Layout {
    /// Frame dimensions of last frame, used to handle navigation.
    last_area: Rect,

    /// Layout columns.
    columns: Vec<Column>,
}

impl Layout {
    /// Create a new empty view, with a single default column and no elements.
    pub fn new() -> Self {
        Self {
            last_area: Rect::new(0, 0, 0, 0),
            columns: vec![Column::new()],
        }
    }

    /// Calculate ratatui layout for the view's columns.
    fn layout(&self, state: &State) -> ratatui::layout::Layout {
        ratatui::layout::Layout::new(
            Direction::Horizontal,
            self.columns.iter().map(|e| e.width(state)),
        )
    }

    /// Calculate minimum width of the layout.
    fn width(&self, state: &State) -> u16 {
        let mut width = 0;
        for col in &self.columns {
            match col.width(state) {
                Constraint::Min(w)
                | Constraint::Max(w)
                | Constraint::Length(w) => width += w,
                _ => {}
            }
        }
        width
    }

    /// Iterator across pairs of (column, area) for rendering or position
    /// calculation.
    fn iter_layout(
        &self,
        state: &State,
        area: Rect,
    ) -> impl Iterator<Item = (&Column, Rect)> {
        let areas = self.layout(state).split(area).to_vec();
        self.columns.iter().zip(areas.into_iter())
    }

    /// Clamp the provided selected element to fall into valid selection
    /// indices.
    fn clamp_selected(
        &self,
        (col, row): SelectedEl,
        state: &State,
    ) -> SelectedEl {
        let col = col.min(self.columns.len().saturating_sub(1));
        let row = if let Some(column) = self.columns.get(col) {
            row.min(column.child_count(state).saturating_sub(1))
        } else {
            0
        };
        (col, row)
    }

    /// Move the provided current position in the direction indicated by the
    /// provided navigation.
    pub fn navigate(
        &self,
        state: &State,
        current: SelectedEl,
        nav: Navigation,
    ) -> SelectedEl {
        match nav {
            Navigation::Up => self.up(current, state),
            Navigation::Down => self.down(current, state),
            Navigation::Left => self.left(current, state),
            Navigation::Right => self.right(current, state),
        }
    }

    /// Move the selection up one element.
    fn up(&self, (col, row): SelectedEl, state: &State) -> SelectedEl {
        self.clamp_selected((col, row.saturating_sub(1)), state)
    }

    /// Move the selection down one element.
    fn down(&self, (col, row): SelectedEl, state: &State) -> SelectedEl {
        self.clamp_selected((col, row + 1), state)
    }

    /// Move the selection left one column.
    fn left(&self, (col, row): SelectedEl, state: &State) -> SelectedEl {
        let layout: Vec<(&Column, Rect)> =
            self.iter_layout(state, self.last_area).collect();
        let y = if let Some((current_column, current_area)) = layout.get(col) {
            current_column.child_y(*current_area, state, row)
        } else {
            0
        };

        let new_col = col.saturating_sub(1);
        let new_row = if let Some((new_column, new_area)) = layout.get(new_col)
        {
            new_column.child_at_y(*new_area, state, y)
        } else {
            0
        };
        self.clamp_selected((new_col, new_row), state)
    }

    /// Move the selection right one column.
    fn right(&self, (col, row): SelectedEl, state: &State) -> SelectedEl {
        let layout: Vec<(&Column, Rect)> =
            self.iter_layout(state, self.last_area).collect();
        let y = if let Some((current_column, current_area)) = layout.get(col) {
            current_column.child_y(*current_area, state, row)
        } else {
            0
        };

        let new_col = (col + 1).min(self.columns.len().saturating_sub(1));
        let new_row = if let Some((new_column, new_area)) = layout.get(new_col)
        {
            new_column.child_at_y(*new_area, state, y)
        } else {
            0
        };
        self.clamp_selected((new_col, new_row), state)
    }

    /// Render the view into the provided frame based on the state,
    /// highlighting the selected element.
    pub fn render(
        &mut self,
        frame: &mut Frame,
        state: &State,
        (col, row): (usize, usize),
    ) {
        self.last_area = frame.area();
        let areas = self.layout(state).split(self.last_area);
        for (i, (&area, column)) in
            areas.iter().zip(self.columns.iter()).enumerate()
        {
            let selected_index = if col == i { Some(row) } else { None };
            column.render(frame, area, state, selected_index);
        }
    }

    /// Add an element to the last column of the view.
    pub fn add_el(&mut self, el: Box<dyn ElSimp>) {
        if let Some(column) = self.columns.last_mut() {
            column.elements.push(El::Simple(el));
        }
    }

    /// Add an element group to the last column of the view.
    pub fn add_group(&mut self, group: Box<dyn ElGroup>) {
        if let Some(column) = self.columns.last_mut() {
            column.elements.push(El::Group(group));
        }
    }

    /// Add a new column to the view.
    pub fn add_column(&mut self) {
        self.columns.push(Column {
            elements: Vec::new(),
        });
    }
}
