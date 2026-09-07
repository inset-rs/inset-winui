//! `ToggleSwitch` with a header, with on and off content, and disabled.
use crate::{GalleryState, column, section};
use reveal_foundation::{App, Handle};
use reveal_widgets::*;
use reveal_winui::*;

pub fn build(state: Handle<GalleryState>, app: &App, resources: &ThemeResources) -> Vec<WidgetRef> {
    let (wifi, airplane) = (app.get(state).wifi, app.get(state).airplane);
    section(
        "ToggleSwitch",
        resources,
        column(
            vec![
                ToggleSwitch::new(wifi, move |app, on| state.set_state(app, |s| s.wifi = on))
                    .header(Text::new("Wi-Fi"))
                    .into_widget(),
                ToggleSwitch::new(airplane, move |app, on| {
                    state.set_state(app, |s| s.airplane = on)
                })
                .on_content(Text::new("Airplane mode on"))
                .off_content(Text::new("Airplane mode off"))
                .into_widget(),
                ToggleSwitch::new(true, |_, _| {})
                    .is_enabled(false)
                    .header(Text::new("Locked"))
                    .into_widget(),
            ],
            8.0,
        ),
    )
}
