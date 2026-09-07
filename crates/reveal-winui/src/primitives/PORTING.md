# reveal-winui/src/primitives
WinUI home: `ContentPresenter`, `Border`, `BrushTransition`, `VisualStateManager` and the system focus visual, in dxaml/xcp
Ported against: aa3207e6

## common_states.rs → `ButtonBase`

- Change: physical activation keys are stopped before the application's default activation shortcuts, including repeats and Enter when `AcceptsReturn` is false.
  Reason: framework — reveal's global shortcuts otherwise turn those key events into `ActivateIntent` invocations.
  Affect: rejected Enter events and held activation-key repeats do not reach ancestor shortcut handlers.

- Change: `CommonStates` uses reveal's native tap recognition instead of WinUI's pointer capture lifecycle.
  Reason: framework — reveal arbitrates competing gestures through its gesture arena, and retaining that mechanism is an accepted tradeoff for this small interaction difference.
  Affect: moving beyond the tap threshold cancels activation even if the pointer returns inside before release, and focus is requested on recognized tap-down rather than necessarily on the raw press.

## color_transition.rs → `BrushTransition`

- Change: `ColorTransition` fades linearly.
  Reason: os — the transition names a duration; the easing is the compositor's.
  Affect: the 83 ms background fade is linear.

## focus_visual.rs → the system focus visual

- Change: `FocusVisual`'s colours are fixed to black and white.
  Reason: os — XAML takes them from the system colours.
  Affect: matches Windows defaults; ignores a high-contrast palette.
