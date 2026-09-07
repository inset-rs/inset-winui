# reveal-winui/src/primitives
WinUI home: `ContentPresenter`, `Border`, `BrushTransition`, `VisualStateManager` and the system focus visual, in dxaml/xcp
Ported against: aa3207e6

## color_transition.rs → `BrushTransition`

- Change: `ColorTransition` fades linearly.
  Reason: os — the transition names a duration; the easing is the compositor's.
  Affect: the 83 ms background fade is linear.

## focus_visual.rs → the system focus visual

- Change: `FocusVisual`'s colours are fixed to black and white.
  Reason: os — XAML takes them from the system colours.
  Affect: matches Windows defaults; ignores a high-contrast palette.
