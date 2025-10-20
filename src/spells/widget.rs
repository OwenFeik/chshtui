use ratatui::layout::{Constraint, Direction};

use crate::{SheetState, spells::Spell, view};

struct SpellEl<S: AsRef<Spell>> {
    spell: S,
}

impl<S: AsRef<Spell>> view::ElSimp<SheetState> for SpellEl<S> {
    fn dimensions(&self) -> view::Dims {
        todo!()
    }

    fn render(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        state: &SheetState,
        selected: bool,
    ) {
        let [
            title_line,
            traits_and_traditions,
            targeting_and_duration,
            body,
            publication,
        ] = ratatui::layout::Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1), // Name, rarity, rank
                Constraint::Length(1), // Traits and traditions
                Constraint::Length(1), // Target, range, time, duration, sustd
                Constraint::Fill(1),   // Body
                Constraint::Length(1), // Publication
            ],
        )
        .areas(area);

        let spell = self.spell.as_ref();

        frame.render_widget(
            format!(
                "{} (Spell {}) ({:?})",
                &spell.name, &spell.rank, &spell.rarity
            ),
            area,
        );
    }
}
