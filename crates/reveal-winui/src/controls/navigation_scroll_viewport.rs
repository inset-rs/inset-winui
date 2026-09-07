//! NavigationView's vertical scrolling presenter with source ScrollBar chrome over native input.

use crate::*;
use reveal_embedder::{Color, Radius};
use reveal_foundation::{App, Handle, Listener, Timer};
use reveal_painting::{Axis, EdgeInsetsGeometry, TextStyle};
use reveal_widgets::*;
use std::{rc::Rc, time::Duration};

/// Native vertical viewport used by NavigationView's main and footer item areas.
#[derive(Clone, Debug)]
pub(crate) struct NavigationScrollViewport {
    /// Caller-owned scroll state, also used by the native scrollbar drag recognizer.
    controller: Handle<ScrollViewportController>,
    /// Intrinsically sized vertical content.
    child: WidgetRef,
}

impl NavigationScrollViewport {
    /// Wraps content without changing controller ownership or native focus scrolling.
    pub(crate) fn new<K>(
        controller: Handle<ScrollViewportController>,
        child: impl IntoWidget<K>,
    ) -> Self {
        Self {
            controller,
            child: child.into_widget(),
        }
    }
}

/// Native hover lifetime and delayed source expansion state.
pub(crate) struct NavigationScrollViewportState {
    /// Widget attachment.
    state: StateData<NavigationScrollViewport>,
    /// The pointer is inside the scrolling area.
    hovered: bool,
    /// The source Expanded visual state.
    expanded: bool,
    /// Delayed expand/contract callback, canceled on another crossing.
    timer: Option<Timer>,
}

impl StatefulWidget for NavigationScrollViewport {
    type State = NavigationScrollViewportState;
    fn create_state(&self) -> Self::State {
        NavigationScrollViewportState {
            state: StateData::new(),
            hovered: false,
            expanded: false,
            timer: None,
        }
    }
}

impl NavigationScrollViewportState {
    /// Applies the source delayed bar-hover state, leaving scrolling to native widgets.
    fn hover_bar(self: Handle<Self>, app: &mut App, enter: bool) {
        if let Some(timer) = app.get_mut(self).timer.take() {
            timer.cancel(app);
        }
        let timer = Timer::new(
            app,
            if enter {
                SCROLL_BAR_EXPAND_BEGIN_TIME
            } else {
                SCROLL_BAR_CONTRACT_BEGIN_TIME
            },
            Listener::new(move |app| {
                if app.contains(self) && self.mounted(app) {
                    self.set_state(app, |s| {
                        s.expanded = enter;
                        s.timer = None;
                    });
                }
            }),
        );
        app.get_mut(self).timer = Some(timer);
    }

    /// ScrollContentPresenter followed by VerticalScrollBar's track and small-change buttons.
    fn template(self: Handle<Self>, app: &mut App, context: BuildContext) -> WidgetRef {
        let widget = self.widget(app).clone();
        let r = ThemeResources::of(app, context).scroll_bar();
        let metrics = widget.controller.metrics(app);
        let overflow = metrics.scrollable_length > 0.0;
        let expanded = app.get(self).expanded && overflow;
        let mut parts = vec![
            ScrollViewport::new(Axis::Vertical, widget.controller, widget.child).into_widget(),
        ];
        if overflow {
            let mut chrome = Vec::new();
            if expanded {
                chrome.push(
                    Positioned::fill(
                        ControlBorder::new(
                            Brush::Acrylic(r.scroll_bar_track_fill),
                            Brush::Solid(Color::new(0)),
                        )
                        .border_thickness(0.0),
                    )
                    .into_widget(),
                );
                for increase in [false, true] {
                    let controller = widget.controller;
                    let button = RepeatButton::new(
                        FluentIcon::new(if increase {
                            FluentSymbol::ChevronDown
                        } else {
                            FluentSymbol::ChevronUp
                        })
                        .font_size(SCROLL_BAR_BUTTON_ARROW_ICON_FONT_SIZE),
                        Listener::new(move |app| {
                            // ScrollViewerLineDelta in ScrollViewer_Partial.h.
                            controller.scroll_by(app, if increase { 16.0 } else { -16.0 }, true);
                        }),
                    )
                    .is_tab_stop(false)
                    .is_enabled(if increase {
                        metrics.offset < metrics.scrollable_length
                    } else {
                        metrics.offset > 0.0
                    })
                    .interval(Duration::from_millis(50))
                    .template(move |app, context, states, content| {
                        arrow_template(app, context, states, content, increase)
                    });
                    let button = Positioned::new(button)
                        .left(0.0)
                        .right(0.0)
                        .height(SCROLL_BAR_SIZE);
                    chrome.push(
                        if increase {
                            button.bottom(0.0)
                        } else {
                            button.top(0.0)
                        }
                        .into_widget(),
                    );
                }
            }
            parts.push(
                Positioned::new(
                    MouseRegion::new()
                        .opaque(false)
                        .on_enter(Rc::new(move |app, _| self.hover_bar(app, true)))
                        .on_exit(Rc::new(move |app, _| self.hover_bar(app, false)))
                        .child(Stack::new().children(chrome)),
                )
                .right(0.0)
                .top(0.0)
                .bottom(0.0)
                .width(SCROLL_BAR_SIZE)
                .into_widget(),
            );
        }
        // ThumbVisual has a six-pixel transparent stroke; RawScrollbar paints only its visible fill.
        let thickness = if expanded {
            SCROLL_BAR_SIZE
        } else {
            SCROLL_BAR_VERTICAL_THUMB_MIN_WIDTH
        } - SCROLL_BAR_THUMB_STROKE_THICKNESS;
        RawScrollbar::new(Stack::new().children(parts))
            .controller(widget.controller.native_controller(app))
            .interactive(true)
            .thumb_visibility(app.get(self).hovered && overflow)
            .thumb_color(r.scroll_bar_thumb_fill)
            .thickness(thickness)
            .radius(Radius::circular(SCROLL_BAR_CORNER_RADIUS[0]))
            .min_thumb_length(
                SCROLL_BAR_VERTICAL_THUMB_MIN_HEIGHT - SCROLL_BAR_THUMB_STROKE_THICKNESS,
            )
            .main_axis_margin(SCROLL_BAR_SIZE + SCROLL_BAR_THUMB_STROKE_THICKNESS / 2.0)
            .cross_axis_margin((SCROLL_BAR_SIZE - thickness) / 2.0)
            .padding(EdgeInsetsGeometry::ZERO)
            .scrollbar_orientation(ScrollbarOrientation::Right)
            .fade_duration(SCROLL_BAR_OPACITY_CHANGE_DURATION)
            .time_to_fade(SCROLL_BAR_CONTRACT_DELAY)
            .into_widget()
    }
}

impl State for NavigationScrollViewportState {
    type Widget = NavigationScrollViewport;
    reveal_widgets::state_accessors!();
    fn dispose(self: Handle<Self>, app: &mut App) {
        if let Some(timer) = app.get_mut(self).timer.take() {
            timer.cancel(app);
        }
    }
    fn build(self: Handle<Self>, app: &mut App, _: BuildContext) -> WidgetRef {
        let controller = self.widget(app).controller;
        MouseRegion::new()
            .opaque(false)
            .on_enter(Rc::new(move |app, _| {
                self.set_state(app, |s| s.hovered = true)
            }))
            .on_exit(Rc::new(move |app, _| {
                self.set_state(app, |s| s.hovered = false)
            }))
            .child(ListenableBuilder::new(
                Rc::new(controller),
                move |app, context, _| self.template(app, context),
            ))
            .into_widget()
    }
}

/// Vertical increment/decrement template CommonStates, without taking keyboard traversal focus.
fn arrow_template(
    app: &mut App,
    context: BuildContext,
    states: ControlStates,
    content: WidgetRef,
    increase: bool,
) -> WidgetRef {
    let r = ThemeResources::of(app, context).scroll_bar();
    let (background, foreground) = match states.common {
        CommonState::Normal => (
            r.scroll_bar_button_background,
            r.scroll_bar_button_arrow_foreground,
        ),
        CommonState::PointerOver => (
            r.scroll_bar_button_background_pointer_over,
            r.scroll_bar_button_arrow_foreground_pointer_over,
        ),
        CommonState::Pressed => (
            r.scroll_bar_button_background_pressed,
            r.scroll_bar_button_arrow_foreground_pressed,
        ),
        CommonState::Disabled => (
            r.scroll_bar_button_background_disabled,
            r.scroll_bar_button_arrow_foreground_disabled,
        ),
    };
    ControlBorder::new(Brush::Solid(background), Brush::Solid(Color::new(0)))
        .border_thickness(0.0)
        .padding(if increase {
            SCROLL_BAR_VERTICAL_INCREASE_MARGIN
        } else {
            SCROLL_BAR_VERTICAL_DECREASE_MARGIN
        })
        .child(Center::new().child(DefaultTextStyle::new(
            TextStyle::new().color(foreground),
            content,
        )))
        .into_widget()
}
