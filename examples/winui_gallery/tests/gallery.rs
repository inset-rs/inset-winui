//! The navigation gallery keeps one feature visible and retains visited demo state.
#![feature(arbitrary_self_types)]
mod common;

use common::Fixture;
use winui_gallery::Feature;

#[test]
fn gallery_navigates_retains_examples_and_switches_theme() {
    let mut fixture = Fixture::new([1080, 780]);
    fixture.capture("gallery_button_light");
    fixture.tap("Standard");
    fixture.tap("Accent");
    fixture.find("Clicked 2 times · wifi true · airplane false");
    fixture.tap("Disabled");
    fixture.find("Clicked 2 times · wifi true · airplane false");

    fixture.navigate(Feature::ToggleSwitch);
    fixture.tap("Wi-Fi");
    fixture.tap("Airplane mode off");
    fixture.find("Clicked 2 times · wifi false · airplane true");

    fixture.navigate(Feature::ToggleButton);
    fixture.tap("Toggle me");
    fixture.find("ToggleButton: checked · three-state unchecked");
    fixture.navigate(Feature::Slider);
    fixture.find("Slider: 42 · stepped 50 · vertical 30");
    fixture.capture("gallery_slider_light");
    fixture.navigate(Feature::ToggleButton);
    fixture.find("ToggleButton: checked · three-state unchecked");

    fixture.tap("Dark theme");
    fixture.find("Light theme");
    fixture.capture("gallery_toggle_button_dark");
    fixture.navigate(Feature::Acrylic);
    fixture.find("Opaque fallback");
    fixture.capture("gallery_acrylic_dark");
    fixture.navigate(Feature::TabView);
    fixture.find("Page 0");
    fixture.capture("gallery_tabs_dark");
    fixture.navigate(Feature::NavigationView);
    fixture.find(Feature::NavigationView.description());
    fixture.capture("gallery_navigation_dark");

    fixture.navigate(Feature::SplitView);
    fixture.tap("Close pane");
    fixture.find("Open pane");
    fixture.tap("Open pane");
    fixture.pump();
    fixture.find("CompactInline · PaneOpened");
    fixture.capture("gallery_split_view_dark");

    fixture.navigate(Feature::Button);
    fixture.find("Clicked 2 times · wifi false · airplane true");
}

#[test]
fn every_feature_has_a_destination_and_small_windows_use_the_pane_toggle() {
    let mut fixture = Fixture::new([1080, 780]);
    for feature in Feature::ALL {
        fixture.navigate(feature);
        fixture.find(feature.description());
    }
    let mut narrow = Fixture::new([640, 720]);
    narrow.find(Feature::Button.description());
    // The compact shell keeps the demo usable with its navigation pane closed.
    narrow.tap("Standard");
    narrow.find("Clicked 1 times · wifi true · airplane false");
    narrow.send(
        reveal_embedder::PointerChange::Down,
        reveal_embedder::Offset::new(24.0, 24.0),
    );
    narrow.send(
        reveal_embedder::PointerChange::Up,
        reveal_embedder::Offset::new(24.0, 24.0),
    );
    narrow.pump();
    narrow.navigate(Feature::Slider);
    narrow.find(Feature::Slider.description());
    narrow.capture("gallery_narrow");
}

#[test]
fn navigation_demo_minimal_header_clears_back_and_toggle_buttons() {
    use reveal_embedder::Offset;
    use reveal_foundation::ValueKey;
    use reveal_rendering::{RenderBox, RenderParagraph};
    use reveal_widgets::{Offstage, downcast_widget};
    use reveal_winui::{NavigationView, NavigationViewPaneDisplayMode};

    let mut f = Fixture::for_feature([1080, 900], Feature::NavigationView);
    f.tap("Mode: Left");
    f.tap("Mode: LeftCompact");
    f.pump();
    let elements = f.elements();
    let app = f.cell.borrow();
    let mut title = None;
    let mut buttons = Vec::new();
    for element in elements {
        let mut parent = Some(element);
        let mut minimal_demo = false;
        let mut visible = true;
        while let Some(ancestor) = parent {
            let widget = ancestor.widget(&app);
            if downcast_widget::<Offstage>(&**widget).is_some_and(|w| w.offstage) {
                visible = false;
                break;
            }
            if let Some(nav) = downcast_widget::<NavigationView>(&**widget) {
                minimal_demo = nav.pane_display_mode == NavigationViewPaneDisplayMode::LeftMinimal;
                break;
            }
            parent = ancestor.parent(&app);
        }
        if !visible || !minimal_demo {
            continue;
        }
        let widget = element.widget(&app);
        if widget.key().is_some_and(|key| {
            ["NavigationViewBackButton", "TogglePaneButton"]
                .iter()
                .any(|name| key.as_ref().eq_key(&ValueKey::new(*name)))
        }) {
            let render = element.find_render_object(&app).unwrap().as_box().unwrap();
            buttons.push(render.local_to_global(&app, Offset::ZERO, None) & render.size(&app));
        }
        if let Some(paragraph) = element
            .render_object(&app)
            .and_then(|r| r.downcast::<RenderParagraph>(&app))
            && paragraph.text(&app).to_plain_text(true, true) == "overview"
        {
            let render = paragraph.as_box();
            title = Some(render.local_to_global(&app, Offset::ZERO, None) & render.size(&app));
        }
    }
    let title = title.expect("the selected page uses NavigationView.Header");
    assert_eq!(buttons.len(), 2);
    for button in buttons {
        assert!(
            title.top >= button.bottom,
            "header {title:?} overlaps button {button:?}"
        );
    }
    drop(app);
    f.capture("gallery_navigation_minimal_header");
}
