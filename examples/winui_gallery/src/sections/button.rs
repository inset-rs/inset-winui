//! `Button` in its three styles, enabled and disabled.
use crate::{GalleryState, row, section};
use reveal_foundation::{Handle, Listener};
use reveal_widgets::*;
use reveal_winui::*;

pub fn build(state: Handle<GalleryState>, resources: &ThemeResources) -> Vec<WidgetRef> {
    let click = Listener::new(move |app| state.set_state(app, |s| s.clicks += 1));
    section(
        "Button",
        resources,
        row(
            vec![
                Button::text("Standard", click.clone()).into_widget(),
                Button::text("Accent", click.clone())
                    .style(ButtonStyle::Accent)
                    .into_widget(),
                Button::text("Subtle", click.clone())
                    .style(ButtonStyle::Subtle)
                    .into_widget(),
                Button::text("Disabled", click.clone())
                    .is_enabled(false)
                    .into_widget(),
                Button::text("Disabled accent", click)
                    .style(ButtonStyle::Accent)
                    .is_enabled(false)
                    .into_widget(),
            ],
            8.0,
        ),
    )
}
