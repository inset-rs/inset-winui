//! `Slider`: horizontal with a header and a value label, stepped with tick marks, disabled, and vertical.
use crate::{column, label, row, section};
use reveal_foundation::{App, Handle};
use reveal_rendering::{CrossAxisAlignment, MainAxisSize};
use reveal_widgets::*;
// `Orientation` is also a `reveal_widgets` name (the media query's); the slider's is XAML's.
use reveal_winui::Orientation;
use reveal_winui::*;

/// The width the horizontal demos are given.
const DEMO_LENGTH: f64 = 300.0;
/// The height the vertical demo is given: the three horizontal ones stacked.
const VERTICAL_DEMO_HEIGHT: f64 = 184.0;

pub fn build(resources: &ThemeResources) -> Vec<WidgetRef> {
    section("Slider", resources, SliderDemo.into_widget())
}

#[derive(Debug)]
struct SliderDemo;

struct SliderDemoState {
    state: StateData<SliderDemo>,
    volume: f64,
    stepped: f64,
    vertical: f64,
}

impl StatefulWidget for SliderDemo {
    type State = SliderDemoState;
    fn create_state(&self) -> SliderDemoState {
        SliderDemoState {
            state: StateData::new(),
            volume: 42.0,
            stepped: 50.0,
            vertical: 30.0,
        }
    }
}

impl State for SliderDemoState {
    type Widget = SliderDemo;
    reveal_widgets::state_accessors!();

    fn build(self: Handle<Self>, app: &mut App, context: BuildContext) -> WidgetRef {
        let resources = ThemeResources::of(app, context);
        let text = resources.common.text_fill_color_primary;
        let (volume, stepped, vertical) = {
            let state = app.get(self);
            (state.volume, state.stepped, state.vertical)
        };
        let value_label = |value: f64| label(format!("{value:.0}"), TextBlockStyle::Body, text);
        let horizontal = column(
            vec![
                row(
                    vec![
                        SizedBox::new()
                            .width(DEMO_LENGTH)
                            .child(
                                Slider::new(volume, move |app, value| {
                                    self.set_state(app, |s| s.volume = value)
                                })
                                .header(Text::new("Volume")),
                            )
                            .into_widget(),
                        value_label(volume),
                    ],
                    12.0,
                ),
                row(
                    vec![
                        SizedBox::new()
                            .width(DEMO_LENGTH)
                            .child(
                                Slider::new(stepped, move |app, value| {
                                    self.set_state(app, |s| s.stepped = value)
                                })
                                .header(Text::new("Stepped"))
                                .step_frequency(10.0)
                                .tick_frequency(10.0)
                                .tick_placement(TickPlacement::Outside),
                            )
                            .into_widget(),
                        value_label(stepped),
                    ],
                    12.0,
                ),
                SizedBox::new()
                    .width(DEMO_LENGTH)
                    .child(
                        Slider::new(70.0, |_, _| {})
                            .header(Text::new("Disabled"))
                            .is_enabled(false),
                    )
                    .into_widget(),
            ],
            8.0,
        );
        let vertical_demo = row(
            vec![
                SizedBox::new()
                    .width(64.0)
                    .height(VERTICAL_DEMO_HEIGHT)
                    .child(
                        Slider::new(vertical, move |app, value| {
                            self.set_state(app, |s| s.vertical = value)
                        })
                        .header(Text::new("Vertical"))
                        .orientation(Orientation::Vertical),
                    )
                    .into_widget(),
                value_label(vertical),
            ],
            12.0,
        );
        column(
            vec![
                Row::new()
                    .main_axis_size(MainAxisSize::Min)
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .spacing(24.0)
                    .children(vec![horizontal, vertical_demo])
                    .into_widget(),
                label(
                    format!("Slider: {volume:.0} · stepped {stepped:.0} · vertical {vertical:.0}"),
                    TextBlockStyle::Body,
                    resources.common.text_fill_color_secondary,
                ),
            ],
            8.0,
        )
    }
}
