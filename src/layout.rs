use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

pub type SceneElementRenderFunc<S> = Box<dyn Fn(&mut Frame, Rect, &S)>;
pub type SceneGroupRenderFunc<S> = Box<dyn Fn(&mut Frame, Rect, &S)>;

pub struct SceneElement<S> {
    width: u16,
    height: Constraint,
    render: SceneElementRenderFunc<S>,
}

impl<S> SceneElement<S> {
    pub fn new(
        width: u16,
        height: Constraint,
        render: SceneElementRenderFunc<S>,
    ) -> Self {
        Self {
            width,
            height,
            render,
        }
    }
}
/*

pub struct SceneGroup<S> {
    width: u16,
    elements: Vec<SceneElement<S>>,
    render: SceneGroupRenderFunc<>
}

impl<S> SceneGroup<S> {
    fn render(&self)
}
*/

enum SceneColumnEntry<S> {
    Single(SceneElement<S>),
    Group(Vec<SceneElement<S>>, SceneElementRenderFunc<S>),
}

pub struct SceneColumn<S> {
    elements: Vec<SceneElement<S>>,
}

impl<S> SceneColumn<S> {
    fn width(&self) -> u16 {
        self.elements.iter().map(|e| e.width).max().unwrap_or(0)
    }

    fn layout(&self) -> Layout {
        Layout::new(Direction::Vertical, self.elements.iter().map(|e| e.height))
    }

    fn render(&self, frame: &mut Frame, area: Rect, state: &S) {
        let areas = self.layout().split(area);
        for (&area, element) in areas.iter().zip(self.elements.iter()) {
            (element.render)(frame, area, state);
        }
    }
}

#[derive(Default)]
pub struct SceneLayout<S> {
    columns: Vec<SceneColumn<S>>,
}

impl<S> SceneLayout<S> {
    fn layout(&self) -> Layout {
        Layout::new(
            Direction::Horizontal,
            self.columns.iter().map(|e| Constraint::Min(e.width())),
        )
    }

    pub fn render(&self, frame: &mut Frame, state: &S) {
        let areas = self.layout().split(frame.area());
        for (&area, column) in areas.iter().zip(self.columns.iter()) {
            column.render(frame, area, state);
        }
    }

    pub fn column(mut self, col: Vec<SceneElement<S>>) -> Self {
        self.columns.push(SceneColumn { elements: col });
        self
    }
}
