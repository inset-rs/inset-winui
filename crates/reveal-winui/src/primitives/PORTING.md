# reveal-winui/src/primitives
Audience: readers familiar with Flutter and Rust; the headings identify the corresponding WinUI code.

WinUI home: `ContentPresenter`, `Border`, `BrushTransition`, `VisualStateManager` and the system focus visual, in dxaml/xcp
Ported against: aa3207e6

## common_states.rs → `ButtonBase`

- Change: `CommonStates` consumes activation keys even when they do not produce a click.
  Reason: framework — Reveal routes keyboard events through nested shortcut handlers, so a rejected key can otherwise activate an enclosing control.
  Affect: Holding an activation key does not repeatedly trigger an enclosing shortcut, and a button configured to reject Enter still consumes that key.

- Change: `CommonStates` handles button presses with Flutter tap recognition instead of keeping all pointer events until release.
  Reason: framework — Reveal lets tap, drag and scroll recognizers compete for the same press, whereas WinUI can capture the pointer directly for the control.
  Affect: Moving too far cancels the click, even if the pointer returns inside the button before release.

- Change: `CommonStates` leaves keyboard focus on the previously focused widget when a button is clicked.
  Reason: framework — Reveal tracks keyboard-focus highlighting separately from clicks, and the kit follows Flutter buttons by leaving focus unchanged on pointer activation.
  Affect: Tab moves focus and shows a focus ring; a mouse click does not move that ring to the clicked button.

## color_transition.rs → `BrushTransition`

- Change: `ColorTransition` fades background colors at a constant rate.
  Reason: os — Windows chooses how the original fade speeds up and slows down, and those parameters are not provided in the source.
  Affect: The 83 ms fade can progress differently from the Windows fade.

## focus_visual.rs → system focus visual

- Change: `FocusVisual` uses fixed black and white colours.
  Reason: os — XAML obtains these colours from the system palette.
  Affect: `FocusVisual` rings ignore high-contrast palette changes.

## fluent_icon.rs → `FontIcon`

- Change: `FluentIcon` uses the bundled Fluent System Icons font for built-in symbols.
  Reason: os — Segoe Fluent Icons ships with Windows.
  Affect: Icons can differ in shape, spacing and alignment from the Windows icons.

## scroll_viewport.rs → `ScrollViewer`

- Change: `ScrollViewport` uses Flutter viewports, controllers and physics for scrolling.
  Reason: framework — WinUI relies on a Windows service for scrolling gestures and motion, while Reveal already supplies Flutter’s scrolling implementation.
  Affect: Scrolling momentum and behavior at the ends follow Flutter, and scroll commands animate over 100 ms.

## anchored_flyout.rs → attached `Flyout` and `FlyoutPresenter`

- Change: `AnchoredFlyout` opens navigation menus in the application’s Overlay and uses a FocusScope.
  Reason: framework — WinUI can create a separate popup window, while a Flutter Overlay draws inside the existing window.
  Affect: A menu cannot extend outside the window, and closing it returns focus to its opening button if that button still exists.

- Change: `AnchoredFlyout` checks its preferred side of the opening button and the opposite side, rather than all four sides.
  Reason: framework — the native custom-layout callback permits one child measurement per pass, while WinUI’s popup-placement code can measure again for several candidate positions.
  Affect: Near a window edge, a menu can appear on a different side or at a different size than on Windows.

- Change: `AnchoredFlyout` opens and closes immediately instead of animating its appearance.
  Reason: os — the source asks Windows to choose the popup animation through `PopupThemeTransition`, without specifying its duration and motion in the template.
  Affect: `AnchoredFlyout` opens and closes immediately.

## margin.rs → `FrameworkElement` layout

- Change: `Margin` preserves fractional spacing instead of snapping its edges to physical pixels.
  Reason: framework — XAML rounds layout measurements using the display scale, while Reveal passes logical-pixel measurements through without that rounding step.
  Affect: At fractional display scales, the spacing can end between physical pixels instead of on a pixel boundary.

## acrylic.rs → `AcrylicBrush`

- Change: `Brush::Acrylic` omits the faint repeating noise image drawn over the blurred background.
  Reason: framework — Reveal’s host interface cannot yet upload the noise image to its renderer, Valo, although that renderer can repeat an uploaded image.
  Affect: `Brush::Acrylic` surfaces lack the source’s fine grain.

- Change: `Brush::Acrylic` does not automatically replace its blurred background with a solid color when the operating system disables visual effects.
  Reason: os — this host does not report Windows battery and advanced-effects policy changes.
  Affect: Applications must select the solid replacement color themselves.

- Change: `Brush::Acrylic` changes its tint immediately when the application supplies a new color.
  Reason: framework — the port stores material settings as values and has no persistent brush object to animate between replacements.
  Affect: `Brush::Acrylic` tint changes jump to the new color instead of fading over 500 ms.

## brush.rs → `Brush`

- Change: `Brush` paints the shape itself instead of returning a Paint for the caller to draw with.
  Reason: framework — acrylic must sample the scene already drawn behind the shape, which Reveal’s renderer, Valo, expresses through a compositing layer rather than an ordinary fill or stroke.
  Affect: Callers ask the helper to paint a rounded rectangle, a border or an oval instead of passing a Paint to Canvas.

## info_bar_panel.rs → `InfoBarPanel`

- Change: `InfoBarPanel` receives the intended height of a single row as an explicit input.
  Reason: framework — WinUI reads this setting from the parent, but Flutter layout constraints describe the space available to a child, not the settings used to size its parent.
  Affect: When using the panel directly, pass the container’s minimum height minus the panel’s top and bottom margins as `parent_min_height`; the full notification control does this automatically.

## retained_flyout.rs → `FlyoutBase` content lifetime

- Change: `RetainedFlyoutHost` keeps closed flyout content mounted in a root OverlayEntry, independently of its opening button.
  Reason: framework — WinUI keeps its content object alive after removing its visuals, while Flutter requires the child to stay mounted to preserve its State.
  Affect: Child State survives closing and reopening from another opening button, but hidden children can still be laid out; Offstage hides them and TickerMode mutes their animation ticks.

- Change: `FlyoutTarget` updates the flyout’s theme or requests closing after the opening widget is removed, waiting until the frame ends.
  Reason: framework — a Flutter widget reports inherited changes and disposal through its lifecycle, and updating the separate overlay must wait until the current build finishes.
  Affect: Applications wrap opening widgets in FlyoutTarget; removing that widget can take another frame to close the flyout and does not dispose the retained child State.

## Deferred

- Navigation flyout opening and closing animations remain deferred. Trigger: A Windows timing measurement or equivalent host transition is available.
- The blurred background’s repeating noise image remains deferred. Trigger: Reveal can upload that image to the renderer.
- Automatic acrylic fallback policy remains deferred. Trigger: A host reports battery and advanced-effects policy changes.
- Animated tint changes remain deferred. Trigger: The background implementation stores the old and new colors and an animation between them.
