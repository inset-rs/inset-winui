# reveal-winui/src/controls
WinUI home: controls/dev/CommonStyles (templates), dxaml/xcp/dxaml/lib (`<Control>_Partial.cpp`), dxaml/xcp/core/core/elements
Ported against: aa3207e6

## every control → dependency properties, routed events

- Change: a control's value is the caller's, and its events are one closure that receives the value the control wants (`ToggleSwitch::new(is_on, toggled)`, `CheckBox::new(is_checked, checked)`, `Slider::new(value, value_changed)`).
  Reason: language — a dependency property the control writes back to has no Rust shape; the caller decides.
  Affect: a handler that ignores the value leaves the control where it is, as a two-way binding to a rejecting setter would.

## grid/ → `Grid`

- Change: `Grid` gives every child its whole cell; a template's alignment or explicit size inside a cell is written as an `Align` or `SizedBox` around the child.
  Reason: framework — alignment is a property of every XAML element; a reveal child sizes itself within its constraints.
  Affect: a child larger than its cell is squashed to it rather than overflowing.

- Change: `Grid` does no layout rounding.
  Reason: framework — XAML snaps row and column sizes to physical pixels; reveal has no layout rounding.
  Affect: star cells can land on fractional pixels where Windows snaps them.

## toggle_switch.rs → `ToggleSwitch`

- Change: `ToggleSwitch`'s knob travels 167 ms on the fast-out-slow-in spline.
  Reason: os — the template's `RepositionThemeAnimation` takes its timing from the OS animation library.
  Affect: the slide may differ from Windows by tens of milliseconds and in curve. Trigger: a recording from a Windows machine.

- Change: `ToggleSwitch`'s drag starts after reveal's touch slop and then catches up; a release inside the slop is the tap.
  Reason: framework — `Thumb` starts its drag on the press, and nothing arbitrates between it and `Tapped`.
  Affect: on touch the knob stays put for the first 18 px, then jumps to the pointer; the toggle decides the same either way.

## radio_button.rs → `RadioButton`

- Change: `RadioButton` has no `GroupName`; it reports its activation and the caller checks it and unchecks the rest.
  Reason: language — XAML's group is a registry the control writes other members' values into.
  Affect: the caller keeps the selected index; a checked button still ignores a click.

- Change: `RadioButton`'s dot keeps its checked stroke under the pointer.
  Reason: framework — two storyboards animate that stroke and XAML keeps whichever ran last; reveal has no storyboard order.
  Affect: on Windows hovering a checked button briefly swaps the one-pixel dot stroke; here it does not.

## check_box.rs → `CheckBox`

- Change: `CheckBox` takes the size its parent gives; the style's left and centre alignment are not applied.
  Reason: framework — alignment is a property of every XAML element.
  Affect: in a stretched column the control, its hit area and its focus ring span the width where Windows hugs the content.

## repeat_button.rs → `RepeatButton`

- Change: `RepeatButton` repeats on the App clock: 500 ms, then every 33 ms.
  Reason: os — XAML's timer is a Win32 `SetTimer`, quantised to the system timer resolution.
  Affect: a held button clicks somewhat more often than on Windows. Trigger: a click-rate measurement there.

- Change: `RepeatButton` decides "pointer over" while held from the pointer's position against its bounds.
  Reason: framework — reveal has no pointer capture, so enter and exit do not reach a control that holds a press.
  Affect: none visible.

## hyperlink_button.rs → `HyperlinkButton`

- Change: `HyperlinkButton` has no `NavigateUri`.
  Reason: os — the URI goes to the Windows shell launcher.
  Affect: a caller opens its link in `click`. Trigger: a host URL launcher.

## slider.rs → `Slider`

- Change: `Slider`'s filled track is a star weight of its grid rather than a width set after layout.
  Reason: framework — reveal builds before it lays out and cannot read a size during the build.
  Affect: the same length, on a fractional pixel where Windows snaps it.

- Change: `Slider` assumes `StepFrequency` 1, `TickFrequency` 0 and `SnapsTo` step values.
  Reason: os — the property defaults live in the generated type table, not the repository.
  Affect: none if the published defaults hold. Trigger: the table, or a Windows machine.

## split_view.rs, split_view_visuals.rs → `SplitView`

- Change: `SplitView` uses reveal's root `Overlay` and native modal tap recognition for light dismissal.
  Reason: framework — reveal supplies `OverlayPortal` and a gesture arena instead of XAML's owned Popup and pointer routing.
  Affect: an open overlay pane requires an `Overlay` ancestor; dismissal happens on a recognized tap, including outside the SplitView bounds, rather than an outer-layer raw pointer press.

- Change: Auto `OpenPaneLength` (`f64::NAN`) updates from the pane's size after layout.
  Reason: framework — reveal builds the template before measuring its children, so `SizeObserver` reports the measured width after the frame.
  Affect: first layout and content-width changes can take one additional frame to settle; pane content stays mounted across explicit/Auto changes and open/closed states.

- Change: pane lifecycle notifications for caller-supplied property changes and outer-overlay updates run after the frame.
  Reason: framework — reveal forbids updating ancestors and showing an overlay while building a child.
  Affect: `PaneOpening` and explicit `PaneClosing` handlers run after the first layout of the new state, followed by `PaneOpened`/`PaneClosed` at transition completion; a light-dismiss `PaneClosing` still runs before the close request and can cancel it, while initial values render settled without lifecycle events.

- Change: `SplitView` uses native focus scopes and restores the saved focus node without a WinUI `FocusState` reason.
  Reason: framework — reveal's focus API represents the target and traversal policy but not XAML's pointer/keyboard/programmatic focus reason.
  Affect: overlay Tab traversal stays in the pane and closing restores the prior target, while focus-highlight decisions follow reveal's native policy.

## Deferred

- `RepeatButton` `ClickMode`: fixed at `Press`. Trigger: a `ClickMode` on `CommonStates`.
- `RadioButton` arrow keys moving focus within a group, wrapping. Trigger: a group scope plus a traversal policy that can be told the candidates.
- The header presenter's lazy load, written as omitting it without a header.
- `ToggleSwitch`'s 83 ms colour fades into pointer-over, pressed and disabled, written discrete. Trigger: a `ColorTransition` whose duration varies per transition.
- `ToggleSwitch` retains a tap target spanning the outer control, including the header, rather than only `SwitchThumb`. Trigger: a review of source-matched pointer hit areas using native reveal gestures.
- `Grid` children with `Visibility="Collapsed"` (zero size, still in the cell), written as omitting the child.
- `Slider`'s value tool tip: the `ToolTip` control is not ported. Trigger: `ToolTip` on an overlay, with the thumb-relative placement of `ToolTipService`.
- `Slider` focus engagement and gamepad keys. Trigger: gamepad or remote input.
- `SplitView` Windows system-back integration, element sounds, Xbox automatic dimming and special cross-pane gamepad XY traversal. Desktop Auto overlay is transparent; Escape and GamepadB dismiss. Trigger: a host supplying those platform facilities.
- `Slider` `LargeChange`: no key uses it in the source; not exposed. Trigger: the automation peer.
- Right-to-left mirroring of templates and of `Slider`'s arrow keys. Trigger: `Directionality` wired through the templates.
- `CheckBox`'s animated sweep into the indeterminate state, written as the bar appearing at once. Trigger: a recording from a Windows machine.
- `ProgressBar`, `ProgressRing`, `TextBlock` styles beyond the ramp, `Expander`, `InfoBar`, `NavigationView`, `TabView`, flyouts and dialogs: not started.
- Mica and Acrylic backdrops: the Acrylic effect graph is in the source; Mica's wallpaper processing is not.
- Text input controls: the editing engine is RichEdit, outside the source.
