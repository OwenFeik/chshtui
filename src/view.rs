use ratatui::{
    Frame,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Margin, Position, Rect},
    widgets::{Block, Clear},
};

use crate::SheetState;

pub type Handler = HandleResult<SheetState>;

pub enum HandleResult<S> {
    Close,
    Open(Box<dyn Scene<S>>),
    Consume,
    Default,
}

/// A scene is a full-screen (potentially floating) view of the application.
/// A scene is expected to track non-application global state internally and
/// update it appropriately based on user into.
pub trait Scene<S> {
    /// Returns a reference to the scene's layout for navigation or rendering.
    fn layout(&self) -> &Layout<S>;

    /// Handle user input entered while the scene is active. Should return
    /// [HandleResult::Consume] if the input was used to update state, or
    /// [HandleResult::Default] if the parent context should handle the event.
    ///
    /// The default implementation ignores all events except keypresses and
    /// delegates keypresses to [Scene::handle_key_press].
    fn handle(
        &mut self,
        event: Event,
        state: &mut S,
        selected: ElPos,
    ) -> HandleResult<S> {
        let el_result = self.layout().handle(event.clone(), state, selected);
        if !matches!(el_result, HandleResult::Default) {
            return el_result;
        }

        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                return self.handle_key_press(key_event.code, state);
            }
        }
        HandleResult::Default
    }

    /// Handle a key press while the scene was active. If the keypress is used
    /// by the scene, [HandleResult::Consume] should be returned.
    /// Otherwise [HandleResult::Default] can be returned to delegate handling
    /// to the parent context.
    ///
    /// Navigation with arrows/hjkl and exit with q are handled by the global
    /// context automatically.
    fn handle_key_press(
        &mut self,
        _key: KeyCode,
        _state: &mut S,
    ) -> HandleResult<S> {
        HandleResult::Default
    }
}

/// Element dimension constraints.
#[derive(Clone, Copy)]
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

    pub fn length(width: u16, height: u16) -> Self {
        Self::new(Constraint::Length(width), Constraint::Length(height))
    }

    pub fn width(&self) -> Constraint {
        self.x
    }

    pub fn height(&self) -> Constraint {
        self.y
    }
}

impl From<(Constraint, Constraint)> for Dims {
    fn from((x, y): (Constraint, Constraint)) -> Dims {
        Dims { x, y }
    }
}

impl From<Dims> for (Constraint, Constraint) {
    fn from(value: Dims) -> Self {
        (value.x, value.y)
    }
}

/// Trait for simple elements, single elements selected as a whole.
pub trait ElSimp<S> {
    /// Return dimension constraints for this element.
    fn dimensions(&self) -> Dims;

    /// Render this element to the frame in the provided area, based on the
    /// current state. If selected is indicated the element should be styled
    /// appropriately.
    fn render(&self, frame: &mut Frame, area: Rect, state: &S, selected: bool);

    /// Handle a keystroke while this is the active element.
    fn handle(&self, event: Event, state: &mut S) -> HandleResult<S> {
        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                return self.handle_key_press(key_event.code, state);
            }
        }
        HandleResult::Default
    }

    /// Handle a key press on this element. By default, delegates to select or
    /// roll method implementations.
    fn handle_key_press(
        &self,
        code: KeyCode,
        state: &mut S,
    ) -> HandleResult<S> {
        match code {
            KeyCode::Enter => self.handle_select(state),
            KeyCode::Char('r') => self.handle_roll(state),
            _ => HandleResult::Default,
        }
    }

    /// Handle user requesting a roll from this element.
    fn handle_roll(&self, _state: &S) -> HandleResult<S> {
        HandleResult::Default
    }

    /// Handle this element being selected.
    fn handle_select(&self, _state: &S) -> HandleResult<S> {
        HandleResult::Default
    }
}

/// Trait for grouped elements, which are rendered as a single element but
/// selected individually. For example a table with individually selectable
/// rows.
pub trait ElGroup<S> {
    /// Return dimensions for the whole element group.
    fn dimensions(&self, state: &S) -> Dims;

    /// Direction the group of elements is arranged in.
    fn direction(&self) -> Direction;

    /// Render this group of elements into the provided area, based on the
    /// current state. If any element in the group is selected, its index
    /// within the group will be provided, otherwise selected will be None.
    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &S,
        selected: Option<usize>,
    );

    /// Handle a keystroke while this is the active element.
    fn handle(
        &self,
        event: Event,
        state: &mut S,
        selected: usize,
    ) -> HandleResult<S> {
        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                return self.handle_key_press(key_event.code, state, selected);
            }
        }
        HandleResult::Default
    }

    fn handle_key_press(
        &self,
        code: KeyCode,
        state: &mut S,
        selected: usize,
    ) -> HandleResult<S> {
        match code {
            KeyCode::Enter => self.handle_select(state, selected),
            KeyCode::Char('r') => self.handle_roll(state, selected),
            _ => HandleResult::Default,
        }
    }

    fn handle_roll(&self, _state: &S, _selected: usize) -> HandleResult<S> {
        HandleResult::Default
    }

    /// Handle a child of this element being selected by the user.
    fn handle_select(&self, _state: &S, _selected: usize) -> HandleResult<S> {
        HandleResult::Default
    }

    /// Return the number of child elements in this group, for selection
    /// handling.
    fn child_count(&self, state: &S) -> usize;

    /// Calculate and return the centre point of the child at the selected
    /// index when this element group is rendered in the provided area.
    fn child_pos(&self, area: Rect, state: &S, selected: usize) -> (u16, u16);

    /// Return the index of the child at the provided (x, y) position if this
    /// element group is rendered in the provided area.
    fn child_at_pos(&self, area: Rect, state: &S, x: u16, y: u16) -> usize;
}

/// Elements which can appear in view columns. Each element is either a simple
/// single element or a group of elements rendered together.
enum El<S> {
    Simple(Box<dyn ElSimp<S>>),
    Group(Box<dyn ElGroup<S>>),
}

impl<S> El<S> {
    /// Return the dimension constraints for this element.
    fn dimensions(&self, state: &S) -> Dims {
        match self {
            Self::Simple(el) => el.dimensions(),
            Self::Group(el) => el.dimensions(state),
        }
    }

    /// Return the number of child elements for this element. For simple
    /// elements this is always just 1. For groups this is the number of
    /// selectable child elements.
    fn row_count(&self, state: &S) -> usize {
        match self {
            Self::Simple(_) => 1,
            Self::Group(group) => {
                if group.direction() == Direction::Vertical {
                    group.child_count(state)
                } else {
                    1
                }
            }
        }
    }
}

/// Compare two ratatui [Constraint]s, ordering such that more constraining
/// constraints are placed first. This is used to prioritise constraints when
/// laying out columns.
fn compare_constraints(a: &Constraint, b: &Constraint) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (Constraint::Min(a), Constraint::Min(b)) => a.cmp(b),
        (Constraint::Max(a), Constraint::Max(b)) => a.cmp(b),
        (Constraint::Length(a), Constraint::Length(b)) => a.cmp(b),
        (Constraint::Percentage(a), Constraint::Percentage(b)) => a.cmp(b),
        (Constraint::Ratio(a1, a2), Constraint::Ratio(b1, b2)) => {
            (*a1 as f32 / *a2 as f32).total_cmp(&(*b1 as f32 / *b2 as f32))
        }
        (Constraint::Fill(a), Constraint::Fill(b)) => a.cmp(b),
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
struct Column<S> {
    elements: Vec<El<S>>,
}

impl<S> Column<S> {
    /// Create a new empty column.
    fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Return a constraint for the width of this column in the overall view.
    /// This will be the most constraining constraint of any child element in
    /// the column.
    fn width(&self, state: &S) -> Constraint {
        self.elements
            .iter()
            .map(|e| e.dimensions(state).x)
            .max_by(compare_constraints)
            .unwrap_or(Constraint::Fill(0))
    }

    /// Returns a ratatui layout for this column to lay out child elements for
    /// rendering.
    fn layout(&self, state: &S) -> ratatui::layout::Layout {
        ratatui::layout::Layout::new(
            Direction::Vertical,
            self.elements.iter().map(|e| e.dimensions(state).y),
        )
    }

    /// Iterate across pairs of element and area in layed-out column for
    /// rendering or position calculation.
    fn iter_layout(
        &self,
        state: &S,
        area: Rect,
    ) -> impl Iterator<Item = (&El<S>, Rect)> {
        let areas = self.layout(state).split(area).to_vec();
        self.elements.iter().zip(areas)
    }

    /// Render the column into the provided area based on the current state.
    /// This will render all elements in the column, top to bottom, with the
    /// selected element appropriately styled.
    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &S,
        selected: Option<ColPos>,
    ) {
        let mut row = selected.map(|p| p.row).unwrap_or(usize::MAX);
        for (element, area) in self.iter_layout(state, area) {
            let row_count = element.row_count(state);
            match element {
                El::Simple(el) => el.render(frame, area, state, row == 0),
                El::Group(group) => {
                    let child_index = match group.direction() {
                        Direction::Vertical => {
                            if row < row_count {
                                Some(row)
                            } else {
                                None
                            }
                        }
                        Direction::Horizontal => {
                            if row == 0 {
                                selected.map(|p| p.row_col)
                            } else {
                                None
                            }
                        }
                    };
                    group.render(frame, area, state, child_index);
                }
            }
            row = row.wrapping_sub(row_count);
        }
    }

    /// Pass an event to handle through to the item at the provided index in
    /// this column. Returns the result of that element handling the event, or
    /// [HandleResult::Default] if the index is invalid.
    fn handle(
        &self,
        event: Event,
        state: &mut S,
        selected: ColPos,
    ) -> HandleResult<S> {
        if let Some((el, child_index)) = self.get_element(selected, state) {
            match el {
                El::Simple(el) => el.handle(event, state),
                El::Group(el) => el.handle(event, state, child_index),
            }
        } else {
            HandleResult::Default
        }
    }

    /// Count the number of selectable elements in this column.
    fn row_count(&self, state: &S) -> usize {
        self.elements.iter().map(|e| e.row_count(state)).sum()
    }

    /// Get element in this column at the provided position.
    fn get_element(&self, pos: ColPos, state: &S) -> Option<(&El<S>, usize)> {
        let mut row = pos.row;
        for element in &self.elements {
            let row_count = element.row_count(state);
            if row < row_count {
                let child_index = match element {
                    El::Simple(_) => 0,
                    El::Group(gp) => match gp.direction() {
                        Direction::Vertical => row,
                        Direction::Horizontal => pos.row_col,
                    },
                };
                return Some((element, child_index));
            }
            row = row.wrapping_sub(row_count);
        }
        None
    }

    /// Clamp the row and row_col of the provided selection so that it points
    /// to an element in this column.
    fn clamp_selected(&self, selected: ColPos, state: &S) -> ColPos {
        let mut pos = ColPos {
            row: selected.row.min(self.row_count(state).saturating_sub(1)),
            row_col: selected.row_col,
        };

        if let Some((el, _)) = self.get_element(pos, state) {
            if let El::Group(gp) = el
                && gp.direction() == Direction::Horizontal
            {
                pos.row_col =
                    pos.row_col.min(gp.child_count(state).saturating_sub(1));
            } else {
                pos.row_col = 0;
            }
        }

        pos
    }

    /// Calculate the position of the element at the provided selection index
    /// within this column when this column is rendered in the provided area.
    fn child_pos(&self, area: Rect, state: &S, pos: ColPos) -> (u16, u16) {
        let mut row = pos.row;
        for (element, area) in self.iter_layout(state, area) {
            let row_count = element.row_count(state);
            if row < row_count {
                return match element {
                    El::Simple(_) => centre_of(area),
                    El::Group(group) => match group.direction() {
                        Direction::Vertical => {
                            group.child_pos(area, state, row)
                        }
                        Direction::Horizontal => {
                            group.child_pos(area, state, pos.row_col)
                        }
                    },
                };
            }
            row = row.saturating_sub(row_count);
        }
        (0, 0)
    }

    /// Calculate the selection index into this column of the provided (x, y)
    /// position.
    fn child_at_coordinate(
        &self,
        area: Rect,
        state: &S,
        x: u16,
        y: u16,
    ) -> ColPos {
        let mut row = 0;
        for (el, el_area) in self.iter_layout(state, area) {
            if el_area.contains(Position::new(el_area.x, y)) {
                return match el {
                    El::Simple(_) => ColPos { row, row_col: 0 },
                    El::Group(group) => match group.direction() {
                        Direction::Vertical => ColPos {
                            row: row + group.child_at_pos(el_area, state, x, y),
                            row_col: 0,
                        },
                        Direction::Horizontal => ColPos {
                            row,
                            row_col: group.child_at_pos(el_area, state, x, y),
                        },
                    },
                };
            }
            row += el.row_count(state);
        }
        ColPos::default()
    }
}

#[derive(Clone, Copy, Default, Debug)]
struct ColPos {
    /// Row in column that contains element.
    row: usize,

    /// Column within row in column that element is.
    row_col: usize,
}

/// Selection coordinate into the view. Notw that this does not just resolve to
/// columns[col].elements[row] because elements in a column may have multiple
/// selected children, or a child may have multiple columns (within parent).
#[derive(Clone, Copy, Default, Debug)]
pub struct ElPos {
    /// Column of selected element.
    col: usize,

    /// Position within column of element.
    pos: ColPos,
}

impl ElPos {
    fn new(col: usize, row: usize, row_col: usize) {
        Self {
            col,
            pos: ColPos { row, row_col },
        }
    }
}

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

/// Describes how to render a layout into a frame.
enum LayoutRenderMode {
    /// Use the whole terminal, spacing elements out across it.
    FullScreen,

    /// Render the layout into a floating centred modal with title and
    /// dimensions.
    Modal {
        title: String,
        dimensions: Dims,
        selection: bool,
    },
}

/// View of the application state. Handles rendering the ratatui TUI based on
/// the current state and the provided elements.
pub struct Layout<S> {
    /// Layout columns.
    columns: Vec<Column<S>>,

    /// Describes how to render the layout into a frame.
    mode: LayoutRenderMode,
}

impl<S> Layout<S> {
    /// Create a new empty view, with a single default column and no elements.
    pub fn new() -> Self {
        Self {
            columns: vec![Column::new()],
            mode: LayoutRenderMode::FullScreen,
        }
    }

    /// Convert this layout into a modal with the provided dimensions.
    pub fn modal(
        mut self,
        title: &str,
        dimensions: Dims,
        selection: bool,
    ) -> Self {
        self.mode = LayoutRenderMode::Modal {
            title: title.to_string(),
            dimensions,
            selection,
        };
        self
    }

    /// Calculate ratatui layout for the view's columns.
    fn layout(&self, state: &S) -> ratatui::layout::Layout {
        ratatui::layout::Layout::new(
            Direction::Horizontal,
            self.columns.iter().map(|e| e.width(state)),
        )
    }

    /// Calculate minimum width of the layout.
    fn width(&self, state: &S) -> u16 {
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
        state: &S,
        area: Rect,
    ) -> impl Iterator<Item = (&Column<S>, Rect)> {
        let areas = self.layout(state).split(area).to_vec();
        self.columns.iter().zip(areas)
    }

    /// Clamp the provided selected element to fall into valid selection
    /// indices.
    fn clamp_selected(&self, selected: ElPos, state: &S) -> ElPos {
        let col = selected.col.min(self.columns.len().saturating_sub(1));
        let pos = self
            .columns
            .get(col)
            .map(|column| column.clamp_selected(selected.pos, state))
            .unwrap_or_default();
        ElPos { col, pos }
    }

    /// Move the provided current position in the direction indicated by the
    /// provided navigation. Current area occupied by this layout required to
    /// calculate relative positions of elements
    pub fn navigate(
        &self,
        area: Rect,
        state: &S,
        current: ElPos,
        nav: Navigation,
    ) -> ElPos {
        match nav {
            Navigation::Up => self.up(current, state),
            Navigation::Down => self.down(current, state),
            Navigation::Left => self.left(current, state, area),
            Navigation::Right => self.right(current, state, area),
        }
    }

    /// Move the selection up one element.
    fn up(&self, mut from: ElPos, state: &S) -> ElPos {
        from.pos.row = from.pos.row.saturating_sub(1);
        self.clamp_selected(from, state)
    }

    /// Move the selection down one element.
    fn down(&self, mut from: ElPos, state: &S) -> ElPos {
        from.pos.row += 1;
        self.clamp_selected(from, state)
    }

    /// Move the selection left one column.
    fn left(&self, mut from: ElPos, state: &S, area: Rect) -> ElPos {
        if from.pos.row_col > 0 {
            from.pos.row_col -= 1;
        } else {
            let layout: Vec<(&Column, Rect)> =
                self.iter_layout(state, area).collect();
            let y = if let Some((current_column, current_area)) =
                layout.get(from.col)
            {
                current_column.child_pos(*current_area, state, from.pos).1
            } else {
                0
            };

            from.col = from.col.saturating_sub(1);
            from.pos =
                if let Some((new_column, new_area)) = layout.get(from.col) {
                    let x = new_area.x + new_area.width - 1; // Right side.
                    new_column.child_at_coordinate(*new_area, state, x, y)
                } else {
                    ColPos::default()
                };
        }

        self.clamp_selected(from, state)
    }

    /// Move the selection right one column.
    fn right(&self, mut from: ElPos, state: &S, area: Rect) -> ElPos {
        // See if we can move to the right within the current column and
        // return early if so.
        if let Some(column) = self.columns.get(from.col) {
            if let Some((El::Group(gp), _)) =
                column.get_element(from.pos, state)
                && gp.direction() == Direction::Horizontal
            {
                if from.pos.row_col + 1 < gp.child_count(state) {
                    from.pos.row_col += 1;
                    return from;
                }
            }
        }

        // Otherwise move right to the same height in the next column.
        let layout: Vec<(&Column, Rect)> =
            self.iter_layout(state, area).collect();
        let y = if let Some((current_column, current_area)) =
            layout.get(from.col)
        {
            current_column.child_pos(*current_area, state, from.pos).1
        } else {
            0
        };

        from.col = from.col + 1;
        from.pos = if let Some((new_column, new_area)) = layout.get(from.col) {
            new_column.child_at_coordinate(*new_area, state, new_area.x, y)
        } else {
            ColPos::default()
        };

        self.clamp_selected(from, state)
    }

    /// Pass an event through to the element at the provided selection location
    /// and return the result of handling it.
    pub fn handle(
        &self,
        event: Event,
        state: &mut S,
        at: ElPos,
    ) -> HandleResult<S> {
        if let Some(column) = self.columns.get(at.col) {
            column.handle(event, state, at.pos)
        } else {
            HandleResult::Default
        }
    }

    /// Render the view into the provided frame based on the state,
    /// highlighting the selected element.
    pub fn render(
        &self,
        frame: &mut Frame,
        state: &S,
        selected: ElPos,
    ) -> Rect {
        let (area, selection) = match &self.mode {
            LayoutRenderMode::FullScreen => (frame.area(), true),
            LayoutRenderMode::Modal {
                title,
                dimensions,
                selection,
            } => {
                let area = centre_in(frame.area(), *dimensions);
                frame.render_widget(Clear, area);
                frame.render_widget(
                    Block::bordered().title(title.as_str()),
                    area,
                );
                (area.inner(Margin::new(1, 1)), *selection)
            }
        };

        for (i, (column, area)) in self.iter_layout(state, area).enumerate() {
            let selected_pos = if selection && selected.col == i {
                Some(selected.pos)
            } else {
                None
            };
            column.render(frame, area, state, selected_pos);
        }

        area
    }

    /// Add an element to the last column of the view.
    pub fn add_el(&mut self, el: Box<dyn ElSimp<S>>) {
        if let Some(column) = self.columns.last_mut() {
            column.elements.push(El::Simple(el));
        }
    }

    /// Add an element group to the last column of the view.
    pub fn add_group(&mut self, group: Box<dyn ElGroup<S>>) {
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

/// Return a box centred within the provided rect, satisfying the provided
/// width and height constraints.
pub fn centre_in(area: Rect, dimensions: Dims) -> Rect {
    let col = ratatui::layout::Layout::new(
        Direction::Vertical,
        [Constraint::Fill(1), dimensions.y, Constraint::Fill(1)],
    );
    let [_above, area, _below] = col.areas(area);
    let row = ratatui::layout::Layout::new(
        Direction::Horizontal,
        [Constraint::Fill(1), dimensions.x, Constraint::Fill(1)],
    );
    let [_left, area, _right] = row.areas(area);
    area
}

/// Return the centre point of the provided rectangle (rounded down).
pub fn centre_of(area: Rect) -> (u16, u16) {
    let x = area.x + (area.width / 2);
    let y = area.y + (area.height / 2);
    (x, y)
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestEl {
        width: u16,
        height: u16,
    }

    impl TestEl {
        fn new(width: u16, height: u16) -> Self {
            Self { width, height }
        }
    }

    impl<S> ElSimp<S> for TestEl {
        fn dimensions(&self) -> Dims {
            Dims::length(self.width, self.height)
        }

        fn render(
            &self,
            frame: &mut Frame,
            area: Rect,
            _state: &S,
            _selected: bool,
        ) {
            frame.render_widget(
                Block::bordered()
                    .title(format!("{}x{}", self.width, self.height)),
                area,
            );
        }
    }

    #[test]
    fn test_centre_in() {
        let area = Rect::new(5, 5, 5, 5);
        let dimensions = Dims::new(Constraint::Length(1), Constraint::Max(1));
        let centre = centre_in(area, dimensions);
        assert_eq!(centre, Rect::new(7, 7, 1, 1));
    }

    #[test]
    fn test_centre_of() {
        let area = Rect::new(1, 1, 5, 10);
        assert_eq!(centre_of(area), (3, 6));
    }

    #[test]
    fn test_navigate() {
        let mut layout = Layout::new();
        layout.add_el(Box::new(TestEl::new(16, 64)));
        layout.add_column();
        layout.add_el(Box::new(TestEl::new(16, 16)));
        layout.add_el(Box::new(TestEl::new(16, 32)));
        layout.add_el(Box::new(TestEl::new(16, 16)));
        layout.add_column();
        layout.add_el(Box::new(TestEl::new(16, 48)));
        layout.add_el(Box::new(TestEl::new(16, 16)));

        // Layout is 48 characters wide and 64 tall. x and y shouldn't matter
        // for navigation.
        let area = Rect::new(128, 128, 16 * 3, 64);

        assert_eq!(
            layout.navigate(area, &(), ElPos::default(), Navigation::Down),
            ElPos::new(0, 1, 0)
        );
    }
}
