//! `CheckBox` unchecked, checked, three-state (starting indeterminate) and disabled, with a label reporting their `IsChecked` values.
use crate::{column, label, section};
use reveal_foundation::{App, Handle};
use reveal_widgets::*;
use reveal_winui::*;

pub fn build(resources: &ThemeResources) -> Vec<WidgetRef> {
    section("CheckBox", resources, CheckBoxDemo.into_widget())
}

#[derive(Debug)]
struct CheckBoxDemo;

struct CheckBoxDemoState {
    state: StateData<CheckBoxDemo>,
    two_state: Option<bool>,
    checked: Option<bool>,
    three_state: Option<bool>,
}

impl StatefulWidget for CheckBoxDemo {
    type State = CheckBoxDemoState;
    fn create_state(&self) -> CheckBoxDemoState {
        CheckBoxDemoState {
            state: StateData::new(),
            two_state: Some(false),
            checked: Some(true),
            three_state: None,
        }
    }
}

fn describe(is_checked: Option<bool>) -> &'static str {
    match is_checked {
        Some(true) => "checked",
        Some(false) => "unchecked",
        None => "indeterminate",
    }
}

impl State for CheckBoxDemoState {
    type Widget = CheckBoxDemo;
    reveal_widgets::state_accessors!();
    fn build(self: Handle<Self>, app: &mut App, context: BuildContext) -> WidgetRef {
        let resources = ThemeResources::of(app, context);
        let (two_state, checked, three_state) = {
            let state = app.get(self);
            (state.two_state, state.checked, state.three_state)
        };
        let status = format!(
            "CheckBox: {} · {} · {}",
            describe(two_state),
            describe(checked),
            describe(three_state)
        );
        column(
            vec![
                CheckBox::new(two_state, move |app, value| {
                    self.set_state(app, |s| s.two_state = value)
                })
                .content(Text::new("Two-state option"))
                .into_widget(),
                CheckBox::new(checked, move |app, value| {
                    self.set_state(app, |s| s.checked = value)
                })
                .content(Text::new("Checked option"))
                .into_widget(),
                CheckBox::new(three_state, move |app, value| {
                    self.set_state(app, |s| s.three_state = value)
                })
                .is_three_state(true)
                .content(Text::new("Three-state option"))
                .into_widget(),
                CheckBox::new(Some(true), |_, _| {})
                    .is_enabled(false)
                    .content(Text::new("Disabled option"))
                    .into_widget(),
                label(
                    status,
                    TextBlockStyle::Body,
                    resources.common.text_fill_color_secondary,
                ),
            ],
            8.0,
        )
    }
}
