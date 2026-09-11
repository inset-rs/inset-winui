//! Source scrollbar presentation backed by native scrolling and drag recognizers.
#![feature(arbitrary_self_types)]
mod common;
#[path = "../../../crates/reveal-winui/src/controls/navigation_scroll_viewport.rs"]
mod presentation;
use common::Fixture;
use presentation::NavigationScrollViewport;
use reveal_embedder::{Offset, PointerChange, PointerData, PointerDataPacket, PointerDeviceKind};
use reveal_foundation::{Handle, Listener};
use reveal_gestures::GestureBinding;
use reveal_rendering::{CrossAxisAlignment, MainAxisSize};
use reveal_widgets::*;
use reveal_winui::*;
use std::{cell::Cell, rc::Rc, time::Duration};
fn fixture(theme: Theme) -> (Fixture, Handle<ScrollViewportController>) {
    let slot = Rc::new(Cell::new(None));
    let q = slot.clone();
    let f = Fixture::with_root([260, 240], move |app| {
        winui_gallery::install_fonts(app);
        let controller = ScrollViewportController::new(app);
        q.set(Some(controller));
        run_app(
            app,
            WidgetsApp::new(AccentPalette::default().base)
                .debug_show_checked_mode_banner(false)
                .builder(move |_, _, _| {
                    let children = (0..30)
                        .map(|i| {
                            SizedBox::new()
                                .height(40.0)
                                .child(Button::text(format!("Item {i}"), Listener::new(|_| {})))
                                .into_widget()
                        })
                        .collect::<Vec<_>>();
                    ThemeScope::new(
                        theme,
                        ColoredBox::new(
                            ThemeResources::new(theme, AccentPalette::default())
                                .common
                                .solid_background_fill_color_base,
                        )
                        .child(NavigationScrollViewport::new(
                            controller,
                            Column::new()
                                .main_axis_size(MainAxisSize::Min)
                                .cross_axis_alignment(CrossAxisAlignment::Stretch)
                                .children(children),
                        )),
                    )
                    .into_widget()
                })
                .into_widget(),
        );
    });
    (f, slot.get().unwrap())
}
fn settle(f: &mut Fixture) {
    for _ in 0..4 {
        f.pump();
    }
}
fn wait(f: &mut Fixture, millis: u64) {
    f.at += Duration::from_millis(millis);
    f.cell.elapse(Duration::from_millis(millis));
    settle(f);
}
#[test]
fn expanded_source_arrows_repeat_scroll_changes() {
    for theme in [Theme::Light, Theme::Dark] {
        let (mut f, c) = fixture(theme);
        settle(&mut f);
        f.send_mouse(PointerChange::Hover, Offset::new(254.0, 230.0), 0);
        wait(&mut f, 650);
        f.capture(if theme == Theme::Light {
            "navigation-scrollbar-light"
        } else {
            "navigation-scrollbar-dark"
        });
        f.send_mouse(PointerChange::Down, Offset::new(254.0, 234.0), 1);
        wait(&mut f, 800);
        f.send_mouse(PointerChange::Up, Offset::new(254.0, 234.0), 0);
        settle(&mut f);
        assert!(
            c.metrics(&f.cell.borrow()).offset >= 32.0,
            "repeat arrow failed: {:?}",
            c.metrics(&f.cell.borrow())
        );
    }
}
#[test]
fn native_thumb_drag_and_track_press_change_the_same_controller() {
    let (mut f, c) = fixture(Theme::Light);
    settle(&mut f);
    f.send_mouse(PointerChange::Hover, Offset::new(254.0, 24.0), 0);
    wait(&mut f, 650);
    f.send_mouse(PointerChange::Down, Offset::new(254.0, 28.0), 1);
    for y in [45.0, 65.0, 90.0] {
        let mut app = f.cell.borrow_mut();
        GestureBinding::instance(&mut app).handle_pointer_data_packet(
            &mut app,
            PointerDataPacket::new(vec![PointerData {
                change: PointerChange::Move,
                kind: PointerDeviceKind::Mouse,
                pointer_identifier: f.mouse_pointer,
                physical_x: 254.0,
                physical_y: y,
                physical_delta_y: 20.0,
                buttons: 1,
                time_stamp: f.at,
                ..Default::default()
            }]),
        );
        drop(app);
        f.cell.checkpoint();
        settle(&mut f);
    }
    f.send_mouse(PointerChange::Up, Offset::new(254.0, 90.0), 0);
    settle(&mut f);
    assert!(
        c.metrics(&f.cell.borrow()).offset > 100.0,
        "thumb drag failed: {:?}",
        c.metrics(&f.cell.borrow())
    );
    c.change_view(&mut f.cell.borrow_mut(), 0.0, true);
    settle(&mut f);
    f.send_mouse(PointerChange::Down, Offset::new(254.0, 180.0), 1);
    f.send_mouse(PointerChange::Up, Offset::new(254.0, 180.0), 0);
    wait(&mut f, 300);
    assert!(
        c.metrics(&f.cell.borrow()).offset > 100.0,
        "native page track failed"
    );
}
