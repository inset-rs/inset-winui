//! A working consumer of the WinUI kit: every control in its styles and states, in the light and dark themes.
#![feature(arbitrary_self_types)]
use reveal_cupertino::CupertinoApp;
use reveal_embedder::Color;
use reveal_foundation::{App, Handle, Listener};
use reveal_painting::{EdgeInsetsGeometry, PaintingBinding};
use reveal_rendering::{CrossAxisAlignment, MainAxisSize};
use reveal_widgets::*;
use reveal_winui::*;

/// Registers the bundled Selawik faces under the kit's font family.
pub fn install_fonts(app: &mut App) {
    let painting = PaintingBinding::instance(app);
    for bytes in [
        &include_bytes!("../fonts/selawk.ttf")[..],
        &include_bytes!("../fonts/selawkb.ttf")[..],
        &include_bytes!("../fonts/selawksb.ttf")[..],
        &include_bytes!("../fonts/selawkl.ttf")[..],
        &include_bytes!("../fonts/selawksl.ttf")[..],
    ] {
        painting.register_font(app, FONT_FAMILY, bytes.to_vec());
    }
}

pub fn run(app: &mut App) {
    install_fonts(app);
    run_app(
        app,
        CupertinoApp::new()
            .debug_show_checked_mode_banner(false)
            .title("WinUI Gallery")
            .home(Gallery)
            .into_widget(),
    );
}

#[derive(Debug)]
pub struct Gallery;
pub struct GalleryState {
    state: StateData<Gallery>,
    dark: bool,
    clicks: u32,
    wifi: bool,
    airplane: bool,
}
impl StatefulWidget for Gallery {
    type State = GalleryState;
    fn create_state(&self) -> GalleryState {
        GalleryState {
            state: StateData::new(),
            dark: false,
            clicks: 0,
            wifi: true,
            airplane: false,
        }
    }
}

pub(crate) fn label(text: impl Into<String>, style: TextBlockStyle, color: Color) -> WidgetRef {
    Text::new(text).style(style.text_style(color)).into_widget()
}
pub(crate) fn column(children: Vec<WidgetRef>, spacing: f64) -> WidgetRef {
    Column::new()
        .main_axis_size(MainAxisSize::Min)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(spacing)
        .children(children)
        .into_widget()
}
pub(crate) fn row(children: Vec<WidgetRef>, spacing: f64) -> WidgetRef {
    Row::new()
        .main_axis_size(MainAxisSize::Min)
        .spacing(spacing)
        .children(children)
        .into_widget()
}

impl State for GalleryState {
    type Widget = Gallery;
    reveal_widgets::state_accessors!();
    fn build(self: Handle<Self>, app: &mut App, _context: BuildContext) -> WidgetRef {
        let state = app.get(self);
        let (dark, clicks, wifi, airplane) = (state.dark, state.clicks, state.wifi, state.airplane);
        let theme = if dark { Theme::Dark } else { Theme::Light };
        let resources = ThemeResources::new(theme, AccentPalette::default());
        let text = resources.common.text_fill_color_primary;
        let secondary = resources.common.text_fill_color_secondary;
        let click = Listener::new(move |app| self.set_state(app, |s| s.clicks += 1));
        let toggle_theme = Listener::new(move |app| self.set_state(app, |s| s.dark = !s.dark));
        let body = column(
            vec![
                label("WinUI Gallery", TextBlockStyle::Title, text),
                label(
                    format!("Clicked {clicks} times · wifi {wifi} · airplane {airplane}"),
                    TextBlockStyle::Body,
                    secondary,
                ),
                label("Button", TextBlockStyle::Subtitle, text),
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
                        Button::text("Disabled accent", click.clone())
                            .style(ButtonStyle::Accent)
                            .is_enabled(false)
                            .into_widget(),
                    ],
                    8.0,
                ),
                label("ToggleSwitch", TextBlockStyle::Subtitle, text),
                column(
                    vec![
                        ToggleSwitch::new(wifi, move |app, on| {
                            self.set_state(app, |s| s.wifi = on)
                        })
                        .header(Text::new("Wi-Fi"))
                        .into_widget(),
                        ToggleSwitch::new(airplane, move |app, on| {
                            self.set_state(app, |s| s.airplane = on)
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
                label("Theme", TextBlockStyle::Subtitle, text),
                Button::text(
                    if dark { "Light theme" } else { "Dark theme" },
                    toggle_theme,
                )
                .into_widget(),
            ],
            16.0,
        );
        let page = ColoredBox::new(resources.common.solid_background_fill_color_base).child(
            SingleChildScrollView::new()
                .child(Padding::new(EdgeInsetsGeometry::all(36.0)).child(body)),
        );
        ThemeScope::new(theme, page).into_widget()
    }
}
