//! `RadioButton`: activation checks an unchecked button and only then raises `Checked`; a checked or disabled one ignores the tap; both themes are captured.
#![feature(arbitrary_self_types)]
mod common;
use common::Fixture;

#[test]
fn radio_button_checks_once_per_group_and_switches_theme() {
    let mut fixture = Fixture::new([900, 2000]);
    fixture.find("RadioButton: Option 2");
    fixture.find("Checked 0 times");
    fixture.tap("Option 1");
    fixture.find("RadioButton: Option 1");
    fixture.find("Checked 1 times");
    fixture.tap("Option 1");
    fixture.find("RadioButton: Option 1");
    fixture.find("Checked 1 times");
    fixture.tap("Option 3");
    fixture.find("RadioButton: Option 3");
    fixture.find("Checked 2 times");
    fixture.tap("Disabled");
    fixture.find("RadioButton: Option 3");
    fixture.find("Checked 2 times");
    fixture.capture("radio_button_light");
    fixture.tap("Dark theme");
    fixture.find("Light theme");
    fixture.capture("radio_button_dark");
}
