use ratatui::{
    layout::{Constraint, Direction},
    text::ToLine,
    widgets::Paragraph,
};

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
        let spell = self.spell.as_ref();

        let mut prefix_lines = Vec::new();
        if !spell.time.is_empty() {
            prefix_lines.push(format!("Cast: {}", &spell.time));
        }
        let mut targeting = String::new();
        if !spell.range.is_empty() {
            targeting.push_str("Range: ");
            targeting.push_str(&spell.range);
        }
        if !spell.target.is_empty() {
            targeting.push_str("Targets: ");
            targeting.push_str(&spell.target);
        }
        if !targeting.is_empty() {
            prefix_lines.push(targeting);
        }
        let mut duration = String::new();
        if !spell.duration.is_empty() {
            duration.push_str("Duration: ");
            duration.push_str(&spell.duration);
        }
        if spell.sustained {
            duration.push_str(" (sustained)");
        }
        if !duration.is_empty() {
            prefix_lines.push(duration);
        }

        let [
            title_line,
            publication,
            traits_and_traditions,
            targeting_and_duration,
            body,
        ] = ratatui::layout::Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1), // Name, rarity, rank
                Constraint::Length(1), // Publication
                Constraint::Length(1), // Traits and traditions
                // Target, range, time, duration, sustained
                Constraint::Length(prefix_lines.len() as u16),
                Constraint::Fill(1), // Body
            ],
        )
        .areas(area);

        frame.render_widget(
            format!(
                "{} (Spell {}) ({:?})",
                &spell.name, &spell.rank, &spell.rarity
            ),
            title_line,
        );
        frame.render_widget(spell.publication.to_line(), publication);
        frame.render_widget(
            Paragraph::new(
                prefix_lines
                    .iter()
                    .map(|s| s.to_line())
                    .collect::<Vec<ratatui::text::Line>>(),
            ),
            targeting_and_duration,
        );
        frame.render_widget(
            format!(
                "{} | {}",
                spell.traits.join(", "),
                spell.traditions.join(", ")
            ),
            traits_and_traditions,
        );
    }
}
