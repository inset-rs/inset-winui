//! Fluent symbol sizes, inherited colors and directional mirroring.

use crate::{column, example, example_row, label, section};
use reveal_embedder::TextDirection;
use reveal_foundation::{App, Handle, Listener};
use reveal_rendering::{CrossAxisAlignment, MainAxisSize};
use reveal_widgets::*;
use reveal_winui::*;

/// Named symbols available to controls and gallery navigation.
const SYMBOLS: [(FluentSymbol, &str); 26] = [
    (FluentSymbol::CursorClick, "Cursor click"),
    (FluentSymbol::ToggleLeft, "Toggle"),
    (FluentSymbol::CheckboxChecked, "Checked"),
    (FluentSymbol::CheckboxIndeterminate, "Mixed"),
    (FluentSymbol::RadioButton, "Radio button"),
    (FluentSymbol::Grid, "Grid"),
    (FluentSymbol::Options, "Options"),
    (FluentSymbol::PanelLeft, "Side pane"),
    (FluentSymbol::Tab, "Tab"),
    (FluentSymbol::TooltipQuote, "Tooltip"),
    (FluentSymbol::Layer, "Layers"),
    (FluentSymbol::Link, "Link"),
    (FluentSymbol::ArrowRepeatAll, "Repeat"),
    (FluentSymbol::TextFont, "Text"),
    (FluentSymbol::Apps, "Apps"),
    (FluentSymbol::Add, "Add"),
    (FluentSymbol::Dismiss, "Dismiss"),
    (FluentSymbol::ChevronLeft, "Chevron left"),
    (FluentSymbol::ChevronRight, "Chevron right"),
    (FluentSymbol::ChevronDown, "Chevron down"),
    (FluentSymbol::ChevronUp, "Chevron up"),
    (FluentSymbol::Navigation, "Navigation"),
    (FluentSymbol::Search, "Search"),
    (FluentSymbol::Back, "Back"),
    (FluentSymbol::Settings, "Settings"),
    (FluentSymbol::More, "More"),
];

/// Builds the symbol feature page.
pub fn build(resources: &ThemeResources) -> Vec<WidgetRef> {
    section("Icons", resources, IconDemo.into_widget())
}

/// An interactive symbol catalog.
#[derive(Debug)]
struct IconDemo;

/// Retains icon sizing and foreground choices.
struct IconDemoState {
    /// Native widget linkage.
    state: StateData<IconDemo>,
    /// Position in the sample size list.
    size: usize,
    /// Whether the catalog uses the accent foreground.
    accent: bool,
    /// Whether directional examples use right-to-left layout.
    rtl: bool,
}

impl StatefulWidget for IconDemo {
    type State = IconDemoState;

    fn create_state(&self) -> Self::State {
        IconDemoState {
            state: StateData::new(),
            size: 1,
            accent: false,
            rtl: false,
        }
    }
}

impl State for IconDemoState {
    type Widget = IconDemo;
    reveal_widgets::state_accessors!();

    fn build(self: Handle<Self>, app: &mut App, context: BuildContext) -> WidgetRef {
        let r = ThemeResources::of(app, context);
        let size = [16.0, 20.0, 24.0, 32.0][app.get(self).size];
        let (accent, rtl) = (app.get(self).accent, app.get(self).rtl);
        let foreground = if accent {
            r.accent.base
        } else {
            r.common.text_fill_color_primary
        };
        let options = example_row(
            vec![
                Button::text(
                    format!("Icon size: {size:.0}"),
                    Listener::new(move |app| self.set_state(app, |s| s.size = (s.size + 1) % 4)),
                )
                .into_widget(),
                CheckBox::new(Some(accent), move |app, value| {
                    self.set_state(app, |s| s.accent = value.unwrap_or(false))
                })
                .content(Text::new("Accent icons"))
                .into_widget(),
            ],
            16.0,
        );
        let catalog = LayoutBuilder::new(move |_, _, constraints| {
            let count = (constraints.max_width / 130.0).floor().clamp(1.0, 6.0) as usize;
            Grid::new()
                .column_definitions((0..count).map(|_| ColumnDefinition::new(GridLength::STAR)))
                .row_definitions(
                    (0..SYMBOLS.len().div_ceil(count))
                        .map(|_| RowDefinition::new(GridLength::AUTO)),
                )
                .children(
                    SYMBOLS
                        .into_iter()
                        .enumerate()
                        .map(|(index, (symbol, name))| {
                            GridCell::new(
                                SizedBox::new().height(92.0).child(
                                    Center::new().child(
                                        Column::new()
                                            .main_axis_size(MainAxisSize::Min)
                                            .cross_axis_alignment(CrossAxisAlignment::Center)
                                            .spacing(10.0)
                                            .children([
                                                FluentIcon::new(symbol)
                                                    .font_size(size)
                                                    .foreground(foreground)
                                                    .into_widget(),
                                                label(name, TextBlockStyle::Caption, foreground),
                                            ]),
                                    ),
                                ),
                            )
                            .row(index / count)
                            .column(index % count)
                            .into_widget()
                        }),
                )
                .into_widget()
        })
        .into_widget();
        let direction = column(
            vec![
                CheckBox::new(Some(rtl), move |app, value| {
                    self.set_state(app, |s| s.rtl = value.unwrap_or(false))
                })
                .content(Text::new("Right-to-left icons"))
                .into_widget(),
                Directionality::new(
                    if rtl {
                        TextDirection::Rtl
                    } else {
                        TextDirection::Ltr
                    },
                    example_row(
                        vec![
                            FluentIcon::new(FluentSymbol::Back)
                                .font_size(32.0)
                                .mirrored_when_right_to_left(true)
                                .into_widget(),
                            FluentIcon::new(FluentSymbol::ChevronRight)
                                .font_size(32.0)
                                .mirrored_when_right_to_left(true)
                                .into_widget(),
                        ],
                        24.0,
                    ),
                )
                .into_widget(),
            ],
            20.0,
        );
        column(
            vec![
                example(
                    "Symbol catalog",
                    "Change the size and color to compare symbols used by the gallery and controls.",
                    &r,
                    column(vec![options, catalog], 24.0),
                ),
                example(
                    "Directional icons",
                    "Navigation arrows can mirror with the reading direction.",
                    &r,
                    direction,
                ),
            ],
            24.0,
        )
    }
}
