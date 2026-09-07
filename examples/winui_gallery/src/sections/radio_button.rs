//! `RadioButton`: a group of three with one checked, and a disabled checked one.
use crate::{column, label, section};
use reveal_foundation::{App, Handle};
use reveal_widgets::*;
use reveal_winui::*;

pub fn build(resources: &ThemeResources) -> Vec<WidgetRef> {
    section("RadioButton", resources, RadioButtonSection.into_widget())
}

/// The group's selection lives here, as `GroupName` would keep it in XAML.
#[derive(Debug)]
struct RadioButtonSection;

struct RadioButtonSectionState {
    state: StateData<RadioButtonSection>,
    selected: usize,
    checked_count: u32,
}

impl StatefulWidget for RadioButtonSection {
    type State = RadioButtonSectionState;
    fn create_state(&self) -> RadioButtonSectionState {
        RadioButtonSectionState {
            state: StateData::new(),
            selected: 1,
            checked_count: 0,
        }
    }
}

const OPTIONS: [&str; 3] = ["Option 1", "Option 2", "Option 3"];

impl State for RadioButtonSectionState {
    type Widget = RadioButtonSection;
    reveal_widgets::state_accessors!();

    fn build(self: Handle<Self>, app: &mut App, context: BuildContext) -> WidgetRef {
        let resources = ThemeResources::of(app, context);
        let (selected, checked_count) = {
            let state = app.get(self);
            (state.selected, state.checked_count)
        };
        let mut children: Vec<WidgetRef> = OPTIONS
            .iter()
            .enumerate()
            .map(|(index, name)| {
                RadioButton::new(index == selected, move |app| {
                    self.set_state(app, |s| {
                        s.selected = index;
                        s.checked_count += 1;
                    })
                })
                .content(Text::new(*name))
                .into_widget()
            })
            .collect();
        children.push(
            RadioButton::new(true, |_| {})
                .content(Text::new("Disabled"))
                .is_enabled(false)
                .into_widget(),
        );
        children.push(label(
            format!("RadioButton: {}", OPTIONS[selected]),
            TextBlockStyle::Body,
            resources.common.text_fill_color_secondary,
        ));
        children.push(label(
            format!("Checked {checked_count} times"),
            TextBlockStyle::Caption,
            resources.common.text_fill_color_secondary,
        ));
        column(children, 8.0)
    }
}
