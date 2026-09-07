//! The gallery mounted whole: painting, button activation, toggles, disabled controls and both themes.
#![feature(arbitrary_self_types)]
mod common;
use common::Fixture;

#[test]
fn gallery_renders_activates_and_switches_theme() {
    let mut fixture = Fixture::new([900, 2000]);
    fixture.capture("light");
    fixture.tap("Standard");
    fixture.find("Clicked 1 times · wifi true · airplane false");
    fixture.tap("Accent");
    fixture.find("Clicked 2 times · wifi true · airplane false");
    fixture.tap("Disabled");
    fixture.find("Clicked 2 times · wifi true · airplane false");
    fixture.tap("Wi-Fi");
    fixture.find("Clicked 2 times · wifi false · airplane false");
    fixture.tap("Airplane mode off");
    fixture.find("Clicked 2 times · wifi false · airplane true");
    fixture.find("Airplane mode on");
    fixture.tap("Locked");
    fixture.find("Clicked 2 times · wifi false · airplane true");
    fixture.tap("Dark theme");
    fixture.find("Light theme");
    fixture.capture("dark");
}
