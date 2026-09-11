//! TabView collection changes exercise real layout, input, focus and retained pages.
#![feature(arbitrary_self_types)]
mod common;

use common::Fixture;
use reveal_embedder::{Offset, PointerChange, PointerData, PointerDataPacket, PointerDeviceKind};
use reveal_foundation::{App, Handle, Listener};
use reveal_gestures::GestureBinding;
use reveal_services::{
    HardwareKeyboard, KeyDownEvent, KeyEvent, KeyUpEvent, LogicalKeyboardKey, PhysicalKeyboardKey,
};
use reveal_widgets::*;
use reveal_winui::*;
use std::time::Duration;
use std::{cell::Cell, rc::Rc};

/// Gives tests access to the owner without replacing the TabView.
#[derive(Clone, Debug, Default)]
struct Probe(Rc<Cell<Option<Handle<PageState>>>>);

#[derive(Debug)]
struct Page(Probe, usize);

struct PageState {
    state: StateData<Page>,
    items: Vec<usize>,
    selected: Option<usize>,
    disabled: Vec<usize>,
    hidden: Vec<usize>,
    theme: Theme,
    mode: TabViewWidthMode,
    footer: bool,
    reorder: bool,
    drag: bool,
    allow_drop: bool,
    cancel: bool,
    events: Vec<String>,
    custom_header: Option<Rc<GlobalKey>>,
    custom_feedback: bool,
}

impl StatefulWidget for Page {
    type State = PageState;

    fn create_state(&self) -> PageState {
        PageState {
            state: StateData::new(),
            items: (0..self.1).collect(),
            selected: Some(0),
            disabled: Vec::new(),
            hidden: Vec::new(),
            theme: Theme::Light,
            mode: TabViewWidthMode::Equal,
            footer: false,
            reorder: true,
            drag: false,
            allow_drop: true,
            cancel: false,
            events: Vec::new(),
            custom_header: None,
            custom_feedback: false,
        }
    }
}

impl State for PageState {
    type Widget = Page;
    reveal_widgets::state_accessors!();

    fn build(self: Handle<Self>, app: &mut App, _: BuildContext) -> WidgetRef {
        self.widget(app).0.0.set(Some(self));
        let state = app.get(self);
        let tabs = state
            .items
            .iter()
            .map(|id| {
                if *id == 0
                    && let Some(key) = &state.custom_header
                {
                    return TabViewItem::new(
                        id.to_string(),
                        SizedBox::new()
                            .key(key.clone())
                            .child(Text::new("Custom tab")),
                        Center::new().child(Counter(*id)),
                    );
                }
                TabViewItem::text(
                    id.to_string(),
                    format!("Tab {id}"),
                    Center::new().child(Counter(*id)),
                )
                .is_enabled(!state.disabled.contains(id))
                .is_visible(!state.hidden.contains(id))
            })
            .collect();
        let mut tabs = TabView::new(tabs, state.selected, move |app, index| {
            self.set_state(app, |s| s.selected = index);
        })
        .can_reorder_tabs(state.reorder)
        .can_drag_tabs(state.drag)
        .allow_drop_tabs(state.allow_drop)
        .tab_reorder_requested(move |app, args| {
            self.set_state(app, |state| {
                let item = state.items.remove(args.old_index);
                state.items.insert(args.new_index, item);
                state.selected = args.selected_index;
                state
                    .events
                    .push(format!("reorder {} {}", args.old_index, args.new_index));
            });
        })
        .tab_drag_starting(move |app, args| {
            args.cancel = app.get(self).cancel;
            app.get_mut(self).events.push("start".into());
        })
        .tab_drag_completed(move |app, args| {
            app.get_mut(self)
                .events
                .push(format!("complete {}", args.was_accepted))
        })
        .tab_dropped_outside(move |app, _| app.get_mut(self).events.push("outside".into()))
        .tab_width_mode(state.mode)
        .tab_close_requested(Rc::new(move |app, args| {
            self.set_state(app, |s| {
                s.items.remove(args.index);
            });
        }));
        if state.custom_feedback {
            tabs = tabs.tab_drag_feedback_builder(|_, _, _| {
                ColoredBox::new(reveal_embedder::Color::from_argb(255, 255, 0, 255))
                    .child(Text::new("Independent preview"))
                    .into_widget()
            });
        }
        if state.footer {
            tabs = tabs.tab_strip_footer(Text::new("Footer"));
        }
        ThemeScope::new(state.theme, tabs).into_widget()
    }
}

#[derive(Debug)]
struct Counter(usize);

struct CounterState {
    state: StateData<Counter>,
    count: usize,
}

impl StatefulWidget for Counter {
    type State = CounterState;

    fn create_state(&self) -> CounterState {
        CounterState {
            state: StateData::new(),
            count: 0,
        }
    }
}

impl State for CounterState {
    type Widget = Counter;
    reveal_widgets::state_accessors!();

    fn build(self: Handle<Self>, app: &mut App, _: BuildContext) -> WidgetRef {
        Button::text(
            format!("Page {} count {}", self.widget(app).0, app.get(self).count),
            Listener::new(move |app| self.set_state(app, |s| s.count += 1)),
        )
        .into_widget()
    }
}

fn fixture(count: usize) -> (Fixture, Probe) {
    let p = Probe::default();
    let page = p.clone();
    let f = Fixture::with_root([480, 220], move |app| {
        winui_gallery::install_fonts(app);
        let entry = OverlayEntry::new(
            app,
            Rc::new(move |_, _| Page(page.clone(), count).into_widget()),
            false,
            true,
            false,
        );
        run_app(
            app,
            WidgetsApp::new(AccentPalette::default().base)
                .debug_show_checked_mode_banner(false)
                .builder(move |_, _, _| {
                    FocusScope::new(Overlay::new().initial_entries([entry]))
                        .autofocus(true)
                        .into_widget()
                })
                .into_widget(),
        );
    });
    (f, p)
}

fn settle(f: &mut Fixture) {
    for _ in 0..3 {
        f.pump();
    }
}

fn click(f: &mut Fixture, point: Offset) {
    f.send(PointerChange::Down, point);
    f.send(PointerChange::Up, point);
    settle(f);
}

#[test]
fn selection_retains_page_state_and_close_reconciles_collection() {
    let (mut f, p) = fixture(3);
    settle(&mut f);
    let page = f.find("Page 0 count 0");
    click(&mut f, page);
    let second = f.find("Tab 1");
    click(&mut f, second);
    assert_eq!(f.cell.borrow().get(p.0.get().unwrap()).selected, Some(1));
    let first = f.find("Tab 0");
    click(&mut f, first);
    f.find("Page 0 count 1");
    f.capture("tab_view_light");
    p.0.get()
        .unwrap()
        .set_state(&mut f.cell.borrow_mut(), |s| s.theme = Theme::Dark);
    settle(&mut f);
    f.capture("tab_view_dark");
    // Equal tabs use (480 - header 2 - add 36 - edge padding 8) / 3.
    click(&mut f, Offset::new(6.0 + 434.0 / 3.0 - 20.0, 24.0));
    assert_eq!(f.cell.borrow().get(p.0.get().unwrap()).items, vec![1, 2]);
    assert_eq!(f.cell.borrow().get(p.0.get().unwrap()).selected, Some(0));
    f.find("Page 1 count 0");
}

#[test]
fn offscreen_selection_realizes_only_a_window_and_footer_stays_measured() {
    let (mut f, p) = fixture(1000);
    settle(&mut f);
    assert!(f.elements().len() < 2000, "headers must remain virtualized");
    p.0.get().unwrap().set_state(&mut f.cell.borrow_mut(), |s| {
        s.selected = Some(999);
        s.footer = true;
    });
    settle(&mut f);
    let last = f.find("Tab 999");
    assert!(
        last.dx() > 0.0 && last.dx() < 480.0,
        "selected tab was not revealed: {last:?}"
    );
    f.find("Footer");
    f.capture("tab_view_overflow");
}

#[test]
fn compact_and_content_width_modes_layout() {
    let (mut f, p) = fixture(3);
    settle(&mut f);
    for mode in [TabViewWidthMode::Compact, TabViewWidthMode::SizeToContent] {
        p.0.get()
            .unwrap()
            .set_state(&mut f.cell.borrow_mut(), |s| s.mode = mode);
        settle(&mut f);
        f.capture(if mode == TabViewWidthMode::Compact {
            "tab_view_compact"
        } else {
            "tab_view_content"
        });
    }
}

fn move_tab_pointer(f: &mut Fixture, from: Offset, to: Offset) {
    let mut app = f.cell.borrow_mut();
    GestureBinding::instance(&mut app).handle_pointer_data_packet(
        &mut app,
        PointerDataPacket::new(vec![PointerData {
            change: PointerChange::Move,
            kind: PointerDeviceKind::Mouse,
            time_stamp: f.at,
            pointer_identifier: f.mouse_pointer,
            physical_x: to.dx(),
            physical_y: to.dy(),
            physical_delta_x: to.dx() - from.dx(),
            physical_delta_y: to.dy() - from.dy(),
            buttons: 1,
            ..Default::default()
        }]),
    );
    drop(app);
    f.cell.checkpoint();
    f.pump();
}
fn pointer_reorder(f: &mut Fixture, from: Offset, to: Offset) {
    f.send_mouse(PointerChange::Down, from, 1);
    move_tab_pointer(f, from, to);
    f.send_mouse(PointerChange::Up, to, 0);
    settle(f);
}
fn key(f: &mut Fixture, down: bool, physical: PhysicalKeyboardKey, logical: LogicalKeyboardKey) {
    let event = if down {
        KeyEvent::Down(KeyDownEvent::new(physical, logical, Duration::ZERO))
    } else {
        KeyEvent::Up(KeyUpEvent::new(physical, logical, Duration::ZERO))
    };
    let mut app = f.cell.borrow_mut();
    HardwareKeyboard::instance(&mut app).handle_key_event(&mut app, &event);
    drop(app);
    f.cell.checkpoint();
    settle(f);
}
#[test]
fn pointer_reorder_preserves_selection_page_and_emits_no_external_events_by_default() {
    let (mut f, p) = fixture(3);
    settle(&mut f);
    let page = f.find("Page 0 count 0");
    click(&mut f, page);
    let start = f.find("Tab 0");
    let end = Offset::new(419.0, start.dy());
    pointer_reorder(&mut f, start, end);
    let app = f.cell.borrow();
    let state = app.get(p.0.get().unwrap());
    assert_eq!(state.items, vec![1, 2, 0]);
    assert_eq!(state.selected, Some(2));
    assert_eq!(state.events, vec!["reorder 0 2"]);
    drop(app);
    f.find("Page 0 count 1");
    f.capture("tab_view_reordered");
}
#[test]
fn keyboard_reorder_is_scoped_to_header_and_preserves_focused_identity() {
    let (mut f, p) = fixture(3);
    settle(&mut f);
    f.focus("Tab 0");
    key(
        &mut f,
        true,
        PhysicalKeyboardKey::ALT_LEFT,
        LogicalKeyboardKey::ALT_LEFT,
    );
    key(
        &mut f,
        true,
        PhysicalKeyboardKey::SHIFT_LEFT,
        LogicalKeyboardKey::SHIFT_LEFT,
    );
    key(
        &mut f,
        true,
        PhysicalKeyboardKey::ARROW_RIGHT,
        LogicalKeyboardKey::ARROW_RIGHT,
    );
    key(
        &mut f,
        false,
        PhysicalKeyboardKey::ARROW_RIGHT,
        LogicalKeyboardKey::ARROW_RIGHT,
    );
    key(
        &mut f,
        false,
        PhysicalKeyboardKey::SHIFT_LEFT,
        LogicalKeyboardKey::SHIFT_LEFT,
    );
    key(
        &mut f,
        false,
        PhysicalKeyboardKey::ALT_LEFT,
        LogicalKeyboardKey::ALT_LEFT,
    );
    let app = f.cell.borrow();
    let state = app.get(p.0.get().unwrap());
    assert_eq!(state.items, vec![1, 0, 2]);
    assert_eq!(state.selected, Some(1));
}
#[test]
fn drag_start_can_cancel_and_disabled_drop_does_not_reorder() {
    let (mut f, p) = fixture(3);
    settle(&mut f);
    let owner = p.0.get().unwrap();
    owner.set_state(&mut f.cell.borrow_mut(), |state| {
        state.drag = true;
        state.cancel = true;
    });
    settle(&mut f);
    let start = f.find("Tab 0");
    pointer_reorder(&mut f, start, Offset::new(419.0, start.dy()));
    assert_eq!(f.cell.borrow().get(owner).items, vec![0, 1, 2]);
    assert_eq!(f.cell.borrow().get(owner).events, vec!["start"]);
    owner.set_state(&mut f.cell.borrow_mut(), |state| {
        state.cancel = false;
        state.allow_drop = false;
        state.events.clear();
    });
    settle(&mut f);
    let start = f.find("Tab 0");
    pointer_reorder(&mut f, start, Offset::new(419.0, start.dy()));
    assert_eq!(f.cell.borrow().get(owner).items, vec![0, 1, 2]);
    assert_eq!(
        f.cell.borrow().get(owner).events,
        vec!["start", "complete false", "outside"]
    );
}

#[derive(Clone, Debug, Default)]
struct CrossProbe(Rc<Cell<Option<Handle<CrossState>>>>);
#[derive(Debug)]
struct CrossPage(CrossProbe);
struct CrossState {
    state: StateData<CrossPage>,
    left: Vec<usize>,
    right: Vec<usize>,
    completed: Option<bool>,
}
impl StatefulWidget for CrossPage {
    type State = CrossState;
    fn create_state(&self) -> Self::State {
        CrossState {
            state: StateData::new(),
            left: vec![0, 1],
            right: vec![2],
            completed: None,
        }
    }
}
impl State for CrossState {
    type Widget = CrossPage;
    reveal_widgets::state_accessors!();
    fn build(self: Handle<Self>, app: &mut App, _: BuildContext) -> WidgetRef {
        self.widget(app).0.0.set(Some(self));
        let items = |ids: &[usize]| {
            ids.iter()
                .map(|id| {
                    TabViewItem::text(
                        id.to_string(),
                        format!("Cross {id}"),
                        Text::new(format!("Content {id}")),
                    )
                })
                .collect()
        };
        let left = TabView::new(items(&app.get(self).left), Some(0), |_, _| {})
            .can_drag_tabs(true)
            .tab_drag_completed(move |app, args| {
                app.get_mut(self).completed = Some(args.was_accepted)
            });
        let right = TabView::new(items(&app.get(self).right), Some(0), |_, _| {})
            .tab_strip_drag_over(|_, args| args.accepted = true)
            .tab_strip_drop(move |app, args| {
                let id = args.item.id.parse::<usize>().unwrap();
                self.set_state(app, |state| {
                    state.left.retain(|item| *item != id);
                    state.right.insert(args.insertion_index, id);
                });
            });
        ThemeScope::new(
            Theme::Light,
            Column::new().children([
                SizedBox::new().height(100.0).child(left).into_widget(),
                SizedBox::new().height(100.0).child(right).into_widget(),
            ]),
        )
        .into_widget()
    }
}
#[test]
fn cross_strip_drop_is_owner_managed_and_completes_source_drag() {
    let probe = CrossProbe::default();
    let copy = probe.clone();
    let mut f = Fixture::with_root([480, 220], move |app| {
        winui_gallery::install_fonts(app);
        let entry = OverlayEntry::new(
            app,
            Rc::new(move |_, _| CrossPage(copy.clone()).into_widget()),
            false,
            true,
            false,
        );
        run_app(
            app,
            WidgetsApp::new(AccentPalette::default().base)
                .debug_show_checked_mode_banner(false)
                .builder(move |_, _, _| Overlay::new().initial_entries([entry]).into_widget())
                .into_widget(),
        );
    });
    settle(&mut f);
    let from = f.find("Cross 0");
    let destination = f.find("Cross 2");
    pointer_reorder(
        &mut f,
        from,
        Offset::new(destination.dx() + 140.0, destination.dy()),
    );
    let app = f.cell.borrow();
    let state = app.get(probe.0.get().unwrap());
    assert_eq!(state.left, vec![1]);
    assert_eq!(state.right, vec![2, 0]);
    assert_eq!(state.completed, Some(true));
}

#[test]
fn custom_keyed_header_drag_keeps_live_subtree_and_accepts_independent_feedback() {
    let (mut f, p) = fixture(3);
    settle(&mut f);
    let owner = p.0.get().unwrap();
    let header_key = Rc::new(GlobalKey::new());
    owner.set_state(&mut f.cell.borrow_mut(), |state| {
        state.custom_header = Some(header_key.clone())
    });
    settle(&mut f);
    let initial = header_key
        .current_context(&mut f.cell.borrow_mut())
        .unwrap();
    let start = f.find("Custom tab");
    pointer_reorder(&mut f, start, Offset::new(419.0, start.dy()));
    assert_eq!(f.cell.borrow().get(owner).items, vec![1, 2, 0]);
    assert_eq!(
        header_key.current_context(&mut f.cell.borrow_mut()),
        Some(initial)
    );
    owner.set_state(&mut f.cell.borrow_mut(), |state| {
        state.custom_feedback = true
    });
    settle(&mut f);
    let start = f.find("Custom tab");
    let end = Offset::new(60.0, start.dy());
    f.send_mouse(PointerChange::Down, start, 1);
    move_tab_pointer(&mut f, start, end);
    f.find("Independent preview");
    let pixels = f.view.pixels.borrow();
    let sample = ((36 * f.view.size[0] + 48) * 4) as usize;
    assert_eq!(
        &pixels[sample..sample + 4],
        &[255, 0, 255, 255],
        "feedback must paint above the original strip"
    );
    drop(pixels);
    f.capture("tab_view_drag_custom_feedback");
    f.send_mouse(PointerChange::Up, end, 0);
    settle(&mut f);
    assert_eq!(f.cell.borrow().get(owner).items, vec![0, 1, 2]);
    assert_eq!(
        header_key.current_context(&mut f.cell.borrow_mut()),
        Some(initial)
    );
}

#[test]
fn held_edge_drag_scrolls_to_unrealized_tabs() {
    let (mut f, p) = fixture(30);
    settle(&mut f);
    let start = f.find("Tab 0");
    let edge = Offset::new(440.0, start.dy());
    f.send_mouse(PointerChange::Down, start, 1);
    move_tab_pointer(&mut f, start, edge);
    f.cell.elapse(Duration::from_millis(250));
    f.pump();
    f.capture("tab_view_drag_live_reorder");
    // Holding the pointer at the viewport edge keeps native auto-scrolling alive.
    for _ in 0..15 {
        f.pump();
    }
    f.send_mouse(PointerChange::Up, edge, 0);
    settle(&mut f);
    let app = f.cell.borrow();
    let state = app.get(p.0.get().unwrap());
    let moved_to = state.items.iter().position(|id| *id == 0).unwrap();
    assert!(
        moved_to > 10,
        "edge drag must reach beyond initially realized headers: {moved_to}"
    );
    assert_eq!(state.selected, Some(moved_to));
}

fn control_key(
    f: &mut Fixture,
    physical: PhysicalKeyboardKey,
    logical: LogicalKeyboardKey,
    shift: bool,
) {
    key(
        f,
        true,
        PhysicalKeyboardKey::CONTROL_LEFT,
        LogicalKeyboardKey::CONTROL_LEFT,
    );
    if shift {
        key(
            f,
            true,
            PhysicalKeyboardKey::SHIFT_LEFT,
            LogicalKeyboardKey::SHIFT_LEFT,
        );
    }
    key(f, true, physical, logical);
    key(f, false, physical, logical);
    if shift {
        key(
            f,
            false,
            PhysicalKeyboardKey::SHIFT_LEFT,
            LogicalKeyboardKey::SHIFT_LEFT,
        );
    }
    key(
        f,
        false,
        PhysicalKeyboardKey::CONTROL_LEFT,
        LogicalKeyboardKey::CONTROL_LEFT,
    );
}

#[test]
fn control_tab_moves_page_focus_and_close_recovers_adjacent_header_focus() {
    let (mut f, p) = fixture(3);
    settle(&mut f);
    let page = f.find("Page 0 count 0");
    click(&mut f, page);
    f.focus("Page 0 count 1");
    control_key(
        &mut f,
        PhysicalKeyboardKey::TAB,
        LogicalKeyboardKey::TAB,
        false,
    );
    assert_eq!(f.cell.borrow().get(p.0.get().unwrap()).selected, Some(1));
    key(
        &mut f,
        true,
        PhysicalKeyboardKey::SPACE,
        LogicalKeyboardKey::SPACE,
    );
    key(
        &mut f,
        false,
        PhysicalKeyboardKey::SPACE,
        LogicalKeyboardKey::SPACE,
    );
    f.find("Page 1 count 1");
    control_key(
        &mut f,
        PhysicalKeyboardKey::TAB,
        LogicalKeyboardKey::TAB,
        true,
    );
    assert_eq!(f.cell.borrow().get(p.0.get().unwrap()).selected, Some(0));
    key(
        &mut f,
        true,
        PhysicalKeyboardKey::SPACE,
        LogicalKeyboardKey::SPACE,
    );
    key(
        &mut f,
        false,
        PhysicalKeyboardKey::SPACE,
        LogicalKeyboardKey::SPACE,
    );
    f.find("Page 0 count 2");
    let tab = f.find("Tab 1");
    click(&mut f, tab);
    f.focus("Tab 1");
    let adjacent_focus = primary_focus(&mut f.cell.borrow_mut());
    let tab = f.find("Tab 2");
    click(&mut f, tab);
    f.focus("Tab 2");
    control_key(
        &mut f,
        PhysicalKeyboardKey::F4,
        LogicalKeyboardKey::F4,
        false,
    );
    assert_eq!(f.cell.borrow().get(p.0.get().unwrap()).items, vec![0, 1]);
    assert_eq!(f.cell.borrow().get(p.0.get().unwrap()).selected, Some(1));
    assert_eq!(primary_focus(&mut f.cell.borrow_mut()), adjacent_focus);
    // Closing the last selected page while its content owns focus requires the deferred selection update.
    let page = f.find("Page 1 count 1");
    click(&mut f, page);
    f.focus("Page 1 count 2");
    control_key(
        &mut f,
        PhysicalKeyboardKey::F4,
        LogicalKeyboardKey::F4,
        false,
    );
    assert_eq!(f.cell.borrow().get(p.0.get().unwrap()).selected, Some(0));
    key(
        &mut f,
        true,
        PhysicalKeyboardKey::SPACE,
        LogicalKeyboardKey::SPACE,
    );
    key(
        &mut f,
        false,
        PhysicalKeyboardKey::SPACE,
        LogicalKeyboardKey::SPACE,
    );
    f.find("Page 0 count 3");
}

/// Collection reconciliation must finish before the first layout after removal.
#[test]
fn collection_replacement_is_visible_in_the_first_frame() {
    use reveal_scheduler::SchedulerBinding;

    for (initial_selection, removed, disabled, hidden, expected) in [
        (Some(3), 3, vec![], vec![], 2),
        (Some(3), 3, vec![2], vec![0], 1),
        (Some(3), 0, vec![], vec![], 3),
        (None, 1, vec![], vec![], 2),
        (None, 3, vec![2], vec![0], 1),
    ] {
        let (mut fixture, probe) = fixture(4);
        let owner = probe.0.get().unwrap();
        owner.set_state(&mut fixture.cell.borrow_mut(), |state| {
            state.selected = initial_selection;
            state.disabled = disabled;
            state.hidden = hidden;
        });
        settle(&mut fixture);
        owner.set_state(&mut fixture.cell.borrow_mut(), |state| {
            state.items.remove(removed);
        });
        fixture.at += Duration::from_millis(20);
        SchedulerBinding::handle_begin_frame(&mut fixture.cell.borrow_mut(), Some(fixture.at));
        fixture.cell.checkpoint();
        SchedulerBinding::handle_draw_frame(&mut fixture.cell.borrow_mut());
        fixture.cell.checkpoint();
        // No second build is allowed here: selection_changed only queued that frame.
        fixture.find(&format!("Page {expected} count 0"));
        let state = fixture.cell.borrow();
        let selected = state.get(owner).selected.unwrap();
        assert_eq!(state.get(owner).items[selected], expected);
    }
}
