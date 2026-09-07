//! `ToggleButton`, `RepeatButton` and `HyperlinkButton` in the gallery: toggling in two and three states, a held repeat against the App clock, a hyperlink activation, and both themes.
#![feature(arbitrary_self_types)]
mod common;
use common::Fixture;
use reveal_embedder::{Offset, PointerChange};
use std::time::Duration;

/// Lets `duration` pass on the App clock in 20 ms steps (the timers fire from `AppCell::elapse`, not from frames), pumping frames between steps so what the timers changed is built and painted.
fn hold(fixture: &mut Fixture, duration: Duration) {
    let step = Duration::from_millis(20);
    let mut elapsed = Duration::ZERO;
    while elapsed < duration {
        fixture.cell.elapse(step);
        elapsed += step;
        fixture.pump();
    }
}

#[test]
fn button_family_toggles_repeats_navigates_and_switches_theme() {
    let mut fixture = Fixture::for_feature([900, 2000], winui_gallery::Feature::ToggleButton);
    fixture.find("ToggleButton: unchecked · three-state unchecked");

    // `OnToggleImpl`: unchecked → checked → unchecked in two-state mode.
    fixture.tap("Toggle me");
    fixture.find("ToggleButton: checked · three-state unchecked");
    fixture.tap("Toggle me");
    fixture.find("ToggleButton: unchecked · three-state unchecked");

    // With `IsThreeState`: unchecked → checked → indeterminate → unchecked.
    fixture.tap("Three-state");
    fixture.find("ToggleButton: unchecked · three-state checked");
    fixture.tap("Three-state");
    fixture.find("ToggleButton: unchecked · three-state indeterminate");
    fixture.tap("Three-state");
    fixture.find("ToggleButton: unchecked · three-state unchecked");

    // A disabled toggle button ignores the pointer.
    fixture.tap("Disabled checked");
    fixture.find("ToggleButton: unchecked · three-state unchecked");

    fixture.navigate(winui_gallery::Feature::RepeatButton);

    // `ClickMode_Press`: one click on the press, then one at `Delay` (500 ms) and one every `Interval` (33 ms) while held: 700 ms gives repeats at 500, 533, …, 698.
    let hold_me = fixture.find("Hold me");
    fixture.send(PointerChange::Down, hold_me);
    fixture.pump();
    fixture.find("RepeatButton: 1 clicks");
    hold(&mut fixture, Duration::from_millis(700));
    fixture.send(PointerChange::Up, hold_me);
    fixture.pump();
    fixture.find("RepeatButton: 8 clicks");

    // Released: no more clicks however long the clock runs.
    hold(&mut fixture, Duration::from_millis(500));
    fixture.find("RepeatButton: 8 clicks");

    // A disabled repeat button does nothing on the press.
    let disabled = fixture.find("Disabled repeat");
    fixture.send(PointerChange::Down, disabled);
    hold(&mut fixture, Duration::from_millis(600));
    fixture.send(PointerChange::Up, disabled);
    fixture.pump();
    fixture.find("RepeatButton: 8 clicks");

    fixture.navigate(winui_gallery::Feature::HyperlinkButton);
    fixture.tap("Learn more");
    fixture.find("HyperlinkButton: 1 clicks");
    fixture.tap("Disabled link");
    fixture.find("HyperlinkButton: 1 clicks");

    fixture.navigate(winui_gallery::Feature::ToggleButton);

    // Leave the two-state button checked so the captures show the `Checked` row.
    fixture.tap("Toggle me");
    fixture.find("ToggleButton: checked · three-state unchecked");

    fixture.capture("button_family_light");
    fixture.tap("Dark theme");
    fixture.find("Light theme");
    fixture.capture("button_family_dark");
}

/// The same hold with a mouse: the press starts the repeat, the hover before it does not.
#[test]
fn repeat_button_repeats_under_a_mouse() {
    let mut fixture = Fixture::for_feature([900, 2000], winui_gallery::Feature::RepeatButton);
    let hold_me = fixture.find("Hold me");
    fixture.send_mouse(PointerChange::Add, hold_me - Offset::new(50.0, 50.0), 0);
    fixture.send_mouse(PointerChange::Hover, hold_me, 0);
    fixture.pump();
    fixture.find("RepeatButton: 0 clicks");
    fixture.send_mouse(PointerChange::Down, hold_me, 1);
    fixture.pump();
    fixture.find("RepeatButton: 1 clicks");
    hold(&mut fixture, Duration::from_millis(700));
    fixture.send_mouse(PointerChange::Up, hold_me, 0);
    fixture.pump();
    fixture.find("RepeatButton: 8 clicks");
}
