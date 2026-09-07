//! `ToggleButton` (two-state, three-state, disabled unchecked and checked), `RepeatButton` held against a counter, and `HyperlinkButton`; each keeps its demo state in its own widget and reports it in a status label.
use crate::{column, label, row, section};
use reveal_foundation::{App, Handle, Listener};
use reveal_widgets::*;
use reveal_winui::*;

pub fn build(resources: &ThemeResources) -> Vec<WidgetRef> {
    let mut sections = section("ToggleButton", resources, ToggleButtonDemo.into_widget());
    sections.extend(section(
        "RepeatButton",
        resources,
        RepeatButtonDemo.into_widget(),
    ));
    sections.extend(section(
        "HyperlinkButton",
        resources,
        HyperlinkButtonDemo.into_widget(),
    ));
    sections
}

/// The status label under each demo, in the secondary text colour.
fn status(app: &mut App, context: BuildContext, text: String) -> WidgetRef {
    let resources = ThemeResources::of(app, context);
    label(
        text,
        TextBlockStyle::Body,
        resources.common.text_fill_color_secondary,
    )
}

fn describe(is_checked: Option<bool>) -> &'static str {
    match is_checked {
        Some(true) => "checked",
        Some(false) => "unchecked",
        None => "indeterminate",
    }
}

#[derive(Debug)]
struct ToggleButtonDemo;
struct ToggleButtonDemoState {
    state: StateData<ToggleButtonDemo>,
    two_state: Option<bool>,
    three_state: Option<bool>,
}
impl StatefulWidget for ToggleButtonDemo {
    type State = ToggleButtonDemoState;
    fn create_state(&self) -> ToggleButtonDemoState {
        ToggleButtonDemoState {
            state: StateData::new(),
            two_state: Some(false),
            three_state: Some(false),
        }
    }
}
impl State for ToggleButtonDemoState {
    type Widget = ToggleButtonDemo;
    reveal_widgets::state_accessors!();
    fn build(self: Handle<Self>, app: &mut App, context: BuildContext) -> WidgetRef {
        let (two_state, three_state) = (app.get(self).two_state, app.get(self).three_state);
        column(
            vec![
                row(
                    vec![
                        ToggleButton::text("Toggle me", two_state, move |app, value| {
                            self.set_state(app, |s| s.two_state = value)
                        })
                        .into_widget(),
                        ToggleButton::text("Three-state", three_state, move |app, value| {
                            self.set_state(app, |s| s.three_state = value)
                        })
                        .is_three_state(true)
                        .into_widget(),
                        ToggleButton::text("Disabled unchecked", Some(false), |_, _| {})
                            .is_enabled(false)
                            .into_widget(),
                        ToggleButton::text("Disabled checked", Some(true), |_, _| {})
                            .is_enabled(false)
                            .into_widget(),
                    ],
                    8.0,
                ),
                status(
                    app,
                    context,
                    format!(
                        "ToggleButton: {} · three-state {}",
                        describe(two_state),
                        describe(three_state)
                    ),
                ),
            ],
            8.0,
        )
    }
}

#[derive(Debug)]
struct RepeatButtonDemo;
struct RepeatButtonDemoState {
    state: StateData<RepeatButtonDemo>,
    clicks: u32,
}
impl StatefulWidget for RepeatButtonDemo {
    type State = RepeatButtonDemoState;
    fn create_state(&self) -> RepeatButtonDemoState {
        RepeatButtonDemoState {
            state: StateData::new(),
            clicks: 0,
        }
    }
}
impl State for RepeatButtonDemoState {
    type Widget = RepeatButtonDemo;
    reveal_widgets::state_accessors!();
    fn build(self: Handle<Self>, app: &mut App, context: BuildContext) -> WidgetRef {
        let clicks = app.get(self).clicks;
        let click = Listener::new(move |app| self.set_state(app, |s| s.clicks += 1));
        column(
            vec![
                row(
                    vec![
                        RepeatButton::text("Hold me", click).into_widget(),
                        RepeatButton::text("Disabled repeat", Listener::new(|_| {}))
                            .is_enabled(false)
                            .into_widget(),
                    ],
                    8.0,
                ),
                status(app, context, format!("RepeatButton: {clicks} clicks")),
            ],
            8.0,
        )
    }
}

#[derive(Debug)]
struct HyperlinkButtonDemo;
struct HyperlinkButtonDemoState {
    state: StateData<HyperlinkButtonDemo>,
    clicks: u32,
}
impl StatefulWidget for HyperlinkButtonDemo {
    type State = HyperlinkButtonDemoState;
    fn create_state(&self) -> HyperlinkButtonDemoState {
        HyperlinkButtonDemoState {
            state: StateData::new(),
            clicks: 0,
        }
    }
}
impl State for HyperlinkButtonDemoState {
    type Widget = HyperlinkButtonDemo;
    reveal_widgets::state_accessors!();
    fn build(self: Handle<Self>, app: &mut App, context: BuildContext) -> WidgetRef {
        let clicks = app.get(self).clicks;
        let click = Listener::new(move |app| self.set_state(app, |s| s.clicks += 1));
        column(
            vec![
                row(
                    vec![
                        HyperlinkButton::text("Learn more", click).into_widget(),
                        HyperlinkButton::text("Disabled link", Listener::new(|_| {}))
                            .is_enabled(false)
                            .into_widget(),
                    ],
                    8.0,
                ),
                status(app, context, format!("HyperlinkButton: {clicks} clicks")),
            ],
            8.0,
        )
    }
}
