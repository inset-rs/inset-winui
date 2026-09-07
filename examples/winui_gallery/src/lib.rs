//! A working consumer of the WinUI kit: every control in its styles and states, in the light and dark themes.
#![feature(arbitrary_self_types)]
mod sections;

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

/// The kit needs only the widgets layer: `WidgetsApp` supplies the view, media query, focus and text direction, and the gallery is its one page.
pub fn run(app: &mut App) {
    install_fonts(app);
    run_app(
        app,
        WidgetsApp::new(AccentPalette::default().base)
            .title("WinUI Gallery")
            .debug_show_checked_mode_banner(false)
            .builder(|_, _, _| Gallery.into_widget())
            .into_widget(),
    );
}

#[derive(Debug)]
pub struct Gallery;
pub struct GalleryState {
    state: StateData<Gallery>,
    pub dark: bool,
    pub clicks: u32,
    pub wifi: bool,
    pub airplane: bool,
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

/// A section: its heading in the subtitle style, then its content.
pub(crate) fn section(
    title: &str,
    resources: &ThemeResources,
    content: WidgetRef,
) -> Vec<WidgetRef> {
    let text = resources.common.text_fill_color_primary;
    vec![label(title, TextBlockStyle::Subtitle, text), content]
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
        let toggle_theme = Listener::new(move |app| self.set_state(app, |s| s.dark = !s.dark));

        let mut children = vec![
            label("WinUI Gallery", TextBlockStyle::Title, text),
            label(
                format!("Clicked {clicks} times · wifi {wifi} · airplane {airplane}"),
                TextBlockStyle::Body,
                secondary,
            ),
        ];
        children.extend(sections::button::build(self, &resources));
        children.extend(sections::toggle_switch::build(self, app, &resources));
        children.extend(sections::grid::build(&resources));
        children.extend(sections::check_box::build(&resources));
        children.extend(sections::radio_button::build(&resources));
        children.extend(sections::button_family::build(&resources));
        children.extend(sections::slider::build(&resources));
        children.extend(section(
            "Theme",
            &resources,
            Button::text(
                if dark { "Light theme" } else { "Dark theme" },
                toggle_theme,
            )
            .into_widget(),
        ));

        let body = column(children, 16.0);
        let page = ColoredBox::new(resources.common.solid_background_fill_color_base).child(
            SingleChildScrollView::new()
                .child(Padding::new(EdgeInsetsGeometry::all(36.0)).child(body)),
        );
        ThemeScope::new(theme, page).into_widget()
    }
}
