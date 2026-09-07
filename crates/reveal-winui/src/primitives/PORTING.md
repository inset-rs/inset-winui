# reveal-winui/src/primitives
WinUI home: `ContentPresenter`, `Border`, `BrushTransition`, `VisualStateManager` and the system focus visual, in dxaml/xcp
Ported against: aa3207e6

## common_states.rs → `ButtonBase`

- Change: `CommonStates` handles activation keys before application shortcuts, including repeats and rejected Enter presses.
  Reason: framework — Reveal otherwise converts these events into activation intents.
  Affect: `CommonStates` prevents held keys and Enter with `AcceptsReturn` disabled from activating an ancestor shortcut.

- Change: `CommonStates` uses native tap recognition instead of WinUI pointer capture.
  Reason: framework — Reveal resolves competing gestures through its gesture arena.
  Affect: `CommonStates` cancels activation after movement beyond the tap threshold, even after the pointer returns inside, and starts focus on recognized tap-down.

## color_transition.rs → `BrushTransition`

- Change: `ColorTransition` uses linear easing.
  Reason: os — Windows supplies the source transition’s easing.
  Affect: `ColorTransition`’s 83 ms background fade is linear.

## focus_visual.rs → system focus visual

- Change: `FocusVisual` uses fixed black and white colours.
  Reason: os — XAML obtains these colours from the system palette.
  Affect: `FocusVisual` rings ignore high-contrast palette changes.

## fluent_icon.rs → `FontIcon`

- Change: `FluentIcon` uses the bundled Fluent System Icons font for built-in symbols.
  Reason: os — Segoe Fluent Icons ships with Windows.
  Affect: `FluentIcon` glyph shapes and optical metrics differ slightly from Windows.

## scroll_viewport.rs → `ScrollViewer`

- Change: `ScrollViewport` uses Reveal’s native viewports and controllers.
  Reason: framework — Reveal has no XAML DirectManipulation scrolling stack.
  Affect: `ScrollViewport` inertia, overscroll and input motion follow Reveal, including its 100 ms animated scroll commands.

## anchored_flyout.rs → attached `Flyout` and `FlyoutPresenter`

- Change: `AnchoredFlyout` uses a root overlay and native focus scope for navigation flyouts.
  Reason: framework — Reveal has no separate-window XAML popup.
  Affect: `AnchoredFlyout` remains inside the application window and restores focus to a surviving opener when dismissed.

- Change: `AnchoredFlyout` is measured once on its preferred side or the opposite side.
  Reason: framework — Reveal’s layout delegate measures each child once, while WinUI can retry all four sides.
  Affect: `AnchoredFlyout` can choose a different side or size in crowded layouts, with long menus scrolling in the available space.

- Change: `AnchoredFlyout` appears and disappears without a theme transition.
  Reason: os — Windows supplies `PopupThemeTransition` timing.
  Affect: `AnchoredFlyout` opens and closes immediately.

## margin.rs → `FrameworkElement` layout

- Change: `Margin` uses native logical-pixel layout for signed margins.
  Reason: framework — Reveal does not apply XAML’s layout-rounding policy.
  Affect: `Margin` can produce fractional-scale edges that differ from Windows pixel rounding.

## acrylic.rs → `AcrylicBrush`

- Change: `Brush::Acrylic` omits the source’s wrapped noise texture at 2% opacity.
  Reason: framework — Reveal has no host image-upload path for Valo images, although tiled image shaders are available.
  Affect: `Brush::Acrylic` surfaces lack the source’s fine grain.

- Change: `Brush::Acrylic` does not switch automatically to its fallback when system material policy changes.
  Reason: os — this host does not report Windows battery and advanced-effects policy changes.
  Affect: `Brush::Acrylic` callers select the fallback explicitly with `Brush::Solid(recipe.fallback_color)`.

- Change: `Brush::Acrylic` applies a replacement recipe’s tint immediately.
  Reason: language — immutable recipe values do not retain mutable brush-property transition state.
  Affect: `Brush::Acrylic` tint changes do not run the source’s 500 ms transition.

## brush.rs → `Brush`

- Change: `Brush` uses shape-aware fill and stroke methods instead of returning a Paint value.
  Reason: framework — Valo represents backdrop sampling as a layer rather than a paint.
  Affect: `Brush` callers use methods such as `paint_rrect`, `paint_drrect` and `paint_oval` to draw the brush.

## Deferred

- Navigation flyout opening and closing animations remain deferred. Trigger: A Windows timing measurement or equivalent host transition is available.
- Acrylic’s wrapped noise texture remains deferred. Trigger: Host image-upload support for Valo images is available.
- Automatic acrylic fallback policy remains deferred. Trigger: A host reports battery and advanced-effects policy changes.
- Acrylic tint transitions remain deferred. Trigger: Mutable brush-property transition state is available.
