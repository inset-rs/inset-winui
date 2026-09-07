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

- Change: `Grid`'s border thickness and padding are uniform.
  Reason: language — the panel's chrome is one widget rather than four per-side properties.
  Affect: a per-side `BorderThickness` is not expressible yet.

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

## Deferred

- `RepeatButton` `ClickMode`: fixed at `Press`. Trigger: a `ClickMode` on `CommonStates`.
- `RadioButton` arrow keys moving focus within a group, wrapping. Trigger: a group scope plus a traversal policy that can be told the candidates.
- The header presenter's lazy load, written as omitting it without a header.
- `ToggleSwitch`'s 83 ms colour fades into pointer-over, pressed and disabled, written discrete. Trigger: a `ColorTransition` whose duration varies per transition.
- `Grid` children with `Visibility="Collapsed"` (zero size, still in the cell), written as omitting the child.
- `Slider`'s value tool tip: the `ToolTip` control is not ported. Trigger: `ToolTip` on an overlay, with the thumb-relative placement of `ToolTipService`.
- `Slider` focus engagement and gamepad keys. Trigger: gamepad or remote input.
- `Slider` `LargeChange`: no key uses it in the source; not exposed. Trigger: the automation peer.
- Right-to-left mirroring of templates and of `Slider`'s arrow keys. Trigger: `Directionality` wired through the templates.
- `CheckBox`'s animated sweep into the indeterminate state, written as the bar appearing at once. Trigger: a recording from a Windows machine.
- `ProgressBar`, `ProgressRing`, `TextBlock` styles beyond the ramp, `Expander`, `InfoBar`, `NavigationView`, `TabView`, flyouts and dialogs: not started.
- Mica and Acrylic backdrops: the Acrylic effect graph is in the source; Mica's wallpaper processing is not.
- Text input controls: the editing engine is RichEdit, outside the source.
