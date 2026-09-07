//! SplitView's four display modes, pane placement, dimming and cancelable dismissal.
use crate::{column, label, row, section};
use reveal_foundation::{App, Handle, Listener};
use reveal_painting::EdgeInsetsGeometry;
use reveal_widgets::*;
use reveal_winui::*;

pub fn build(resources: &ThemeResources) -> Vec<WidgetRef> {
    section("SplitView", resources, SplitViewDemo.into_widget())
}

#[derive(Debug)]
struct SplitViewDemo;

struct SplitViewDemoState {
    state: StateData<SplitViewDemo>,
    open: bool,
    mode: SplitViewDisplayMode,
    placement: SplitViewPanePlacement,
    dim: bool,
    cancel: bool,
    page: &'static str,
    event: &'static str,
}

impl StatefulWidget for SplitViewDemo {
    type State = SplitViewDemoState;

    fn create_state(&self) -> Self::State {
        SplitViewDemoState {
            state: StateData::new(),
            open: true,
            mode: SplitViewDisplayMode::CompactInline,
            placement: SplitViewPanePlacement::Left,
            dim: true,
            cancel: false,
            page: "Home",
            event: "Ready",
        }
    }
}

impl State for SplitViewDemoState {
    type Widget = SplitViewDemo;
    reveal_widgets::state_accessors!();

    fn build(self: Handle<Self>, app: &mut App, context: BuildContext) -> WidgetRef {
        let resources = ThemeResources::of(app, context);
        let text = resources.common.text_fill_color_primary;
        let (open, mode, placement, dim, cancel, page, event) = {
            let state = app.get(self);
            (
                state.open,
                state.mode,
                state.placement,
                state.dim,
                state.cancel,
                state.page,
                state.event,
            )
        };
        let mode_name = match mode {
            SplitViewDisplayMode::Overlay => "Overlay",
            SplitViewDisplayMode::Inline => "Inline",
            SplitViewDisplayMode::CompactOverlay => "CompactOverlay",
            SplitViewDisplayMode::CompactInline => "CompactInline",
        };
        let controls = row(
            vec![
                Button::text(
                    if open { "Close pane" } else { "Open pane" },
                    Listener::new(move |app| self.set_state(app, |s| s.open = !s.open)),
                )
                .into_widget(),
                Button::text(
                    mode_name,
                    Listener::new(move |app| {
                        self.set_state(app, |s| {
                            s.mode = match s.mode {
                                SplitViewDisplayMode::Overlay => SplitViewDisplayMode::Inline,
                                SplitViewDisplayMode::Inline => {
                                    SplitViewDisplayMode::CompactOverlay
                                }
                                SplitViewDisplayMode::CompactOverlay => {
                                    SplitViewDisplayMode::CompactInline
                                }
                                SplitViewDisplayMode::CompactInline => {
                                    SplitViewDisplayMode::Overlay
                                }
                            };
                        })
                    }),
                )
                .into_widget(),
                Button::text(
                    match placement {
                        SplitViewPanePlacement::Left => "Pane: left",
                        SplitViewPanePlacement::Right => "Pane: right",
                    },
                    Listener::new(move |app| {
                        self.set_state(app, |s| {
                            s.placement = match s.placement {
                                SplitViewPanePlacement::Left => SplitViewPanePlacement::Right,
                                SplitViewPanePlacement::Right => SplitViewPanePlacement::Left,
                            };
                        })
                    }),
                )
                .into_widget(),
            ],
            8.0,
        );
        let options = row(
            vec![
                CheckBox::new(Some(dim), move |app, value| {
                    self.set_state(app, |s| s.dim = value.unwrap_or(false))
                })
                .content(Text::new("Dim content"))
                .into_widget(),
                CheckBox::new(Some(cancel), move |app, value| {
                    self.set_state(app, |s| s.cancel = value.unwrap_or(false))
                })
                .content(Text::new("Cancel light dismiss"))
                .into_widget(),
            ],
            16.0,
        );
        let pane = Padding::new(EdgeInsetsGeometry::all(8.0)).child(column(
            vec![
                label("Pane", TextBlockStyle::BodyStrong, text),
                Button::text(
                    "Home",
                    Listener::new(move |app| self.set_state(app, |s| s.page = "Home")),
                )
                .into_widget(),
                Button::text(
                    "Settings",
                    Listener::new(move |app| self.set_state(app, |s| s.page = "Settings")),
                )
                .into_widget(),
            ],
            8.0,
        ));
        let content = ColoredBox::new(resources.common.solid_background_fill_color_secondary)
            .child(Center::new().child(column(
                vec![
                    label(page, TextBlockStyle::Subtitle, text),
                    label(
                        "Pane and content keep their state.",
                        TextBlockStyle::Caption,
                        text,
                    ),
                ],
                8.0,
            )));
        let split = SplitView::new(open, move |app, value| {
            self.set_state(app, |s| s.open = value)
        })
        .display_mode(mode)
        .pane_placement(placement)
        .open_pane_length(160.0)
        .compact_pane_length(48.0)
        .light_dismiss_overlay_mode(if dim {
            LightDismissOverlayMode::On
        } else {
            LightDismissOverlayMode::Off
        })
        .pane(pane)
        .content(content)
        .pane_opening(Listener::new(move |app| {
            self.set_state(app, |s| s.event = "PaneOpening")
        }))
        .pane_opened(Listener::new(move |app| {
            self.set_state(app, |s| s.event = "PaneOpened")
        }))
        .pane_closing(move |app, args| {
            args.cancel = app.get(self).cancel;
            self.set_state(app, |s| {
                s.event = if s.cancel {
                    "PaneClosing (cancel requested)"
                } else {
                    "PaneClosing"
                }
            });
        })
        .pane_closed(Listener::new(move |app| {
            self.set_state(app, |s| s.event = "PaneClosed")
        }));
        column(
            vec![
                controls,
                options,
                SizedBox::new()
                    .width(620.0)
                    .height(200.0)
                    .child(split)
                    .into_widget(),
                label(
                    format!("{mode_name} · {event}"),
                    TextBlockStyle::Caption,
                    resources.common.text_fill_color_secondary,
                ),
            ],
            8.0,
        )
    }
}
