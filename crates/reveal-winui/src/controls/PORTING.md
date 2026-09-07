# reveal-winui/src/controls
WinUI home: controls/dev/CommonStyles (templates), dxaml/xcp/dxaml/lib (`<Control>_Partial.cpp`), dxaml/xcp/core/core/elements
Ported against: aa3207e6

## mod.rs → caller-owned control values and events

- Change: Public controls keep their values in the caller, and control events report requested values through closures.
  Reason: language — Rust has no XAML dependency-property writeback.
  Affect: A value-change callback that the caller ignores leaves the corresponding control unchanged.

## button.rs → `ControlTemplate`

- Change: `Button` alternate templates are Rust builders receiving state and content.
  Reason: language — Rust uses typed closures instead of XAML template resources.
  Affect: A `Button` caller can supply `.template(...)` without replacing the button’s input behavior.

## grid/render_grid.rs → `Grid`

- Change: `Grid` gives each child its whole grid cell, with alignment expressed by child wrappers.
  Reason: framework — Reveal has no alignment property on every element.
  Affect: `Grid` constrains a child larger than its cell to that cell rather than letting it overflow.

## grid/layout.rs → `Grid` row and column layout

- Change: `Grid` row and column sizes are not rounded to physical pixels.
  Reason: framework — Reveal has no XAML layout-rounding policy.
  Affect: `Grid` star-sized cells can end on fractional pixels.

## toggle_switch.rs → `ToggleSwitch`

- Change: `ToggleSwitch` moves its knob over 167 ms with fast-out-slow-in easing.
  Reason: os — Windows supplies the source `RepositionThemeAnimation` timing.
  Affect: `ToggleSwitch` slide duration and curve can differ from Windows.

- Change: `ToggleSwitch` dragging waits for native touch slop before catching up with the pointer.
  Reason: framework — Reveal arbitrates between tap and drag gestures, while XAML Thumb begins on the press.
  Affect: `ToggleSwitch` keeps its knob still for the first 18 px of touch movement, and shorter movement remains a tap.

## radio_button.rs → `RadioButton`

- Change: `RadioButton` reports activation without managing a `GroupName` registry.
  Reason: language — callers own the values that XAML’s group registry would update.
  Affect: A `RadioButton` caller checks the chosen button and unchecks the others.

- Change: `RadioButton` keeps a checked dot’s stroke while hovered.
  Reason: framework — Reveal does not resolve competing storyboards by their execution order.
  Affect: Hovering a checked `RadioButton` does not briefly swap its one-pixel dot stroke.

## check_box.rs → `CheckBox`

- Change: `CheckBox` takes its parent’s allocated size without applying the style’s alignment.
  Reason: framework — Reveal has no alignment property on every element.
  Affect: In a stretched column, `CheckBox`’s hit area and focus ring span the column.

## repeat_button.rs → `RepeatButton`

- Change: `RepeatButton` uses the App clock with a 500 ms delay and 33 ms interval.
  Reason: os — Windows quantizes its timer to the system timer resolution.
  Affect: A held `RepeatButton` can repeat somewhat faster than on Windows.

## hyperlink_button.rs → `HyperlinkButton`

- Change: `HyperlinkButton` does not expose `NavigateUri`.
  Reason: os — the Windows shell launcher handles that property.
  Affect: A `HyperlinkButton` caller opens the link from the click handler.

## slider.rs → `Slider`

- Change: `Slider`’s filled track uses a grid proportion instead of a width assigned after layout.
  Reason: framework — Reveal builds widgets before their sizes are known.
  Affect: `Slider`’s fill can end on a fractional pixel where Windows rounds it.

- Change: `Slider` thumb tooltips use native ToolTipService and focus-highlight policy.
  Reason: framework — Reveal does not expose WinUI’s focus-reason value.
  Affect: `Slider` tooltips require an Overlay ancestor and use native keyboard-focus, placement and fade behavior.

- Change: `Slider`’s tooltip value converter is a Rust function returning text.
  Reason: language — Rust has no XAML value-converter binding.
  Affect: `Slider` callers provide `thumb_tool_tip_value_converter` for custom text.

- Change: `Slider` vertical tooltip placement assumes a right-handed user.
  Reason: os — this host does not expose Windows handedness settings.
  Affect: `Slider` vertical tooltips prefer the left side of the thumb.

## split_view.rs → `SplitView`

- Change: `SplitView` overlay panes dismiss through native overlay tap recognition.
  Reason: framework — Reveal uses OverlayPortal and gesture arbitration instead of XAML popup pointer routing.
  Affect: `SplitView` requires an Overlay ancestor, and outside dismissal waits for a recognized tap.

- Change: `SplitView` updates automatic pane width after the pane is measured.
  Reason: framework — Reveal cannot read the child’s measured width during widget build.
  Affect: `SplitView` initial layout and content-width changes can take an extra frame to settle.

- Change: `SplitView` property-driven pane lifecycle events run after layout.
  Reason: framework — Reveal forbids updating ancestors or showing overlays during child build.
  Affect: `SplitView` opening and explicit closing notifications arrive after the new layout, while light dismissal remains cancelable before closing.

- Change: `SplitView` pane focus uses native scopes and saved focus nodes.
  Reason: framework — Reveal does not distinguish XAML’s pointer, keyboard and programmatic focus reasons.
  Affect: `SplitView` focus highlighting follows native policy when focus enters or returns from the pane.

## tool_tip/mod.rs → `ToolTip` and `ToolTipService`

- Change: `ToolTipService` uses Reveal’s root overlay and in-window pointer routing.
  Reason: framework — Reveal has no XAML Popup or attached tooltip service.
  Affect: `ToolTipService` callers wrap the owner beneath an Overlay, and the tooltip stays inside that window.

- Change: `ToolTipService` uses a 150 ms fade and the source fallback hover timing.
  Reason: os — this host does not report Windows theme timing or user hover settings.
  Affect: `ToolTipService` initial display waits 800 ms and recent mouse reshow waits 400 ms, which can differ from a configured Windows desktop.

## navigation_scroll_viewport.rs → NavigationView navigation scroll presentation

- Change: `NavigationScrollViewport` combines source scrollbar visuals with native RawScrollbar interaction.
  Reason: framework — Reveal already supplies thumb dragging and track paging.
  Affect: `NavigationScrollViewport` track presses use native page scrolling instead of XAML repeat-button paging.

- Change: `NavigationScrollViewport` switches scrollbar thickness after the source hover delays without a width animation.
  Reason: framework — RawScrollbar exposes thickness but no independent thumb-width animation.
  Affect: `NavigationScrollViewport` thumb widths change at the delayed boundary rather than over 167 ms.

## tab_view.rs → `TabView`

- Change: TabView receives stable-id items and owner-managed selection and collection callbacks.
  Reason: language — immutable widget configuration leaves collection ownership with the caller.
  Affect: Callers apply close, reorder and transfer requests and update the selected index.

- Change: `TabView` presents tab headers through a keyed native ListView with persistent focus handles.
  Reason: framework — Reveal has no XAML item-container generator.
  Affect: Offscreen selection first scrolls to an estimated position and then adjusts to measured headers.

- Change: `TabView` retains previously selected pages with `Offstage` and `TickerMode`, following Flutter’s `CupertinoTabScaffold`.
  Reason: framework — Reveal’s widget State is tied to mounted elements, and Flutter’s tab-switching mechanism preserves that State without a separate content-object lifetime.
  Affect: Inactive `TabView` pages keep their State and have ticker animations muted, but they can still be laid out unlike content detached from WinUI’s presenter.

- Change: `TabView` reconciles selection during a collection update and notifies the owner after the frame.
  Reason: framework — Reveal cannot rebuild an ancestor owner from its child’s widget-update hook.
  Affect: The replacement `TabView` page appears in the current frame, while the owner’s selection callback runs after layout.

## tab_view/drag.rs → TabView tab dragging and reordering

- Change: `TabView` implements tab dragging through native gestures, overlay feedback and edge scrolling.
  Reason: framework — Reveal has no XAML drag-operation stack.
  Affect: Drag thresholds and edge-scroll speed follow Reveal, and transfers stay within one overlay.

- Change: `TabView` moves neighboring headers immediately after the 200 ms reorder delay.
  Reason: os — Windows supplies the source reposition animation.
  Affect: The insertion gap opens without the Windows motion animation.

- Change: `TabView` leaves the destination strip’s layout unchanged during an external drag until the owner inserts the item.
  Reason: framework — native drag targets have no XAML external-item placeholder layout.
  Affect: Cross-strip hover does not widen the strip or open an insertion gap.

## tab_view_item.rs → `TabViewItem`

- Change: Default drag feedback rebuilds text and FluentIcon visuals while omitting arbitrary custom header content.
  Reason: framework — Reveal’s native drag feedback cannot take a bitmap snapshot of the live header.
  Affect: Custom drag visuals need `tab_drag_feedback_builder` with independent keys and focus state.

- Change: Tab headers compose native tap, tooltip and focus primitives.
  Reason: framework — Reveal has no XAML routed pointer capture.
  Affect: Pointer thresholds and focus highlighting follow the native control conventions.

## navigation_indicator_transition.rs → NavigationView selection-indicator animations

- Change: `NavigationIndicatorTransition` starts selection-indicator motion after destination layout.
  Reason: framework — Reveal exposes updated item geometry after the widget layout pass.
  Affect: `NavigationIndicatorTransition` keeps the outgoing item’s indicator until the destination has been measured, which can delay the transition by one frame.

- Change: `NavigationIndicatorTransition` uses native ease-in-out easing for cross-level indicator motion.
  Reason: os — the source leaves that easing to the Windows compositor default.
  Affect: `NavigationIndicatorTransition` cross-level acceleration can differ from Windows.

## navigation_view.rs → `NavigationView`

- Change: `NavigationView` receives stable-id item snapshots and owner-managed selection.
  Reason: language — callers own collection and page state in immutable widget configurations.
  Affect: `NavigationView` callers handle selection and invocation separately and keep ids unique, including the reserved settings id.

- Change: `NavigationView` interaction uses native SplitView lifecycle, focus traversal and activation.
  Reason: framework — Reveal has no XAML routed-event or focus-reason system.
  Affect: `NavigationView` pointer thresholds, keyboard activation timing and lifecycle callbacks follow the native controls.

- Change: `NavigationView` search and badge slots accept caller-supplied widgets.
  Reason: language — these slots do not require separately ported XAML control classes.
  Affect: `NavigationView` lets applications supply their own editor and badge widgets rather than AutoSuggestBox or InfoBadge controls.

- Change: `NavigationView` top navigation applies measured widths to its item partition after layout.
  Reason: framework — Reveal cannot reconcile widgets during XAML-style measurement.
  Affect: `NavigationView`’s primary and overflow split can take an extra frame to settle after content changes.

## navigation_view_item.rs → NavigationView item containers

- Change: `NavigationViewItem` keyed native item trees remain mounted while collapsed or moved between presentations.
  Reason: framework — Reveal preserves widget state through mounted keys rather than XAML element ownership.
  Affect: `NavigationViewItem` preserves custom item state across presentation changes, and its collapsed children remain mounted but cannot paint or take focus.

- Change: `NavigationViewItem` chevrons use Fluent System Icons with a 250 ms native rotation.
  Reason: os — Windows supplies the source symbol font and animated icon.
  Affect: `NavigationViewItem` glyph shapes and intermediate expansion motion differ from the source animated chevron.

## Deferred

- RepeatButton ClickMode remains fixed at Press. Trigger: A consumer needs configurable ClickMode on CommonStates.
- RadioButton arrow-key group traversal remains deferred. Trigger: A group scope supplies traversal candidates.
- `Slider` HeaderContentPresenter lazy loading remains deferred. Trigger: A consumer needs the presenter to remain alive without a header.
- ToggleSwitch state-colour fades remain deferred. Trigger: Transition durations vary by state change.
- ToggleSwitch tap bounds still include the header. Trigger: A source review of hit areas using native gestures is available.
- ToggleSwitch knob animation timing remains unverified. Trigger: A recording from Windows is available.
- Collapsed Grid children are omitted rather than retained at zero size. Trigger: A consumer needs retained collapsed children.
- RepeatButton timer-rate verification remains deferred. Trigger: A Windows click-rate measurement is available.
- HyperlinkButton does not expose NavigateUri. Trigger: A host URL launcher is available.
- Slider default-value verification for StepFrequency, TickFrequency and SnapsTo remains deferred. Trigger: The Windows type table or a runtime measurement is available.
- Slider focus engagement and gamepad input remain deferred. Trigger: A gamepad or remote-input host is available.
- Slider LargeChange remains deferred. Trigger: An automation peer consumes it.
- SplitView system-back handling, element sounds and Xbox focus/dimming behavior remain deferred. Trigger: A host exposes those facilities.
- Template and Slider-key RTL mirroring remain deferred. Trigger: A Directionality pass covers those controls.
- CheckBox indeterminate sweep animation remains deferred. Trigger: A recording from Windows is available.
- ProgressBar, ProgressRing, additional TextBlock styles, Expander, InfoBar and dialogs remain deferred. Trigger: The next control port is selected.
- Mica wallpaper backdrops remain deferred. Trigger: A host material path supplies the wallpaper processing absent from the source.
- Text input controls remain deferred. Trigger: An editing-engine decision addresses the Windows RichEdit dependency.
- TabView tear-out, caption regions, window movement and cross-window drag transport remain deferred. Trigger: A host provides window management and native drag transport.
- TabView gamepad focus, focus engagement, high-contrast adaptation and automation remain deferred. Trigger: The corresponding input, theme and accessibility facilities are available.
- TabView insertion, removal and reorder animations remain deferred. Trigger: Windows timing is verified or a host animation service is available.
- NavigationView title-bar insets, non-client areas, sounds and page-transition execution remain deferred. Trigger: A host exposes those facilities.
- NavigationView gamepad shoulder navigation, XY focus, automation and high-contrast palettes remain deferred. Trigger: The corresponding input, accessibility and theme facilities are available.
- Standalone Flyout, ScrollViewer and ScrollBar APIs remain deferred. Trigger: A consumer needs those public controls beyond NavigationView’s internal adapters.
