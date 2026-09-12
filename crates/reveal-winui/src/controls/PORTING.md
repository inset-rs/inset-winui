# reveal-winui/src/controls
Audience: readers familiar with Flutter and Rust; the headings identify the corresponding WinUI code.

WinUI home: controls/dev/CommonStyles (templates), dxaml/xcp/dxaml/lib (`<Control>_Partial.cpp`), dxaml/xcp/core/core/elements
Ported against: aa3207e6

## mod.rs → shared control conventions

- Change: `Button`, `CheckBox`, `RadioButton`, `ToggleButton`, `HyperlinkButton`, `RepeatButton`, `ToggleSwitch`, `Slider`, and navigation and tab item click handlers do not request keyboard focus on pointer activation.
  Reason: framework — Flutter controls separate pointer activation from focus acquisition, and Reveal's native focus policy is used for this port.
  Affect: These controls acquire focus and its highlight through keyboard traversal; a pointer click leaves any existing keyboard focus highlight in place.

- Change: `ToggleSwitch`, `Slider`, `Expander` and other controls with application-owned values report requested changes through callbacks.
  Reason: framework — WinUI changes values on persistent control objects, while Reveal follows Flutter by rebuilding widgets from application state.
  Affect: A value-change callback that the caller ignores leaves the corresponding control unchanged.

## button.rs → `ControlTemplate`

- Change: `Button` accepts a drawing function to customize its appearance while retaining its input behavior.
  Reason: language — Rust callbacks receive the current button state and content in place of a XAML template resource.
  Affect: A `Button` caller can supply `.template(...)` without replacing the button’s input behavior.

## text_control.rs → TextBoxBase

- Change: `TextBox` and `PasswordBox` use Flutter’s text editor, selection gestures, controllers and input formatters.
  Reason: framework — WinUI delegates editing to Windows RichEdit, while Reveal supplies Flutter's editor.
  Affect: Text selection, keyboard shortcuts, IME composition, length enforcement and undo follow the native Flutter behavior.

- Change: `TextBox` and `PasswordBox` report text and selection changes after the edit has been applied.
  Reason: framework — WinUI can let application code cancel a pending edit or selection change, while Flutter validates edits with input formatters before updating the controller.
  Affect: Applications use input formatters to reject user edits and controller listeners to observe changes, rather than canceling a before-change event.

- Change: `TextControl` adjusts its clear and password-reveal buttons after the editor is measured.
  Reason: framework — the button widgets are built before the editor’s size is known, so they need another build to apply size-dependent changes.
  Affect: A clear or reveal button can settle its visibility or width on the frame following a size change.

## text_box.rs → TextBox

- Change: `TextBox` wraps long lines in multiline mode even when wrapping is disabled.
  Reason: framework — Flutter's multiline layout does not provide independent wrapping and two-axis editor scrolling.
  Affect: A field configured to accept newlines preserves typed line breaks but also wraps lines that exceed its width.

- Change: `TextBox` receives a TextEditingController owned by the application instead of storing editable text on the widget itself.
  Reason: framework — Reveal keeps mutable editing state in a persistent controller while rebuilding the surrounding widget descriptions.
  Affect: Callers read or assign text and selection through the controller and dispose it after the editor unmounts.

## password_box.rs → PasswordBox

- Change: `PasswordBox` keeps its password in a native TextEditingController.
  Reason: os — WinUI's additional CryptProtectMemory storage is a Windows facility outside the native Flutter editing engine.
  Affect: The field conceals text visually and disables copying and cutting, but the controller contains an ordinary text value.

## text_edit_menu.rs → TextCommandBarFlyout desktop commands

- Change: `TextBox` and `PasswordBox` show editing commands in Flutter’s text-selection menu overlay.
  Reason: framework — Flutter’s editor opens, places and closes the menu, whereas WinUI uses a separate popup window for the editing commands.
  Affect: Editing menus stay inside the application window and follow Flutter’s placement and keyboard shortcuts, while preserving WinUI’s command order and rules for enabling commands.

- Change: `TextEditMenu` blurs content inside the application window and appears without an opening animation.
  Reason: os — the Windows system backdrop and popup theme animation are not available through the native overlay host.
  Affect: The menu samples the application scene instead of the desktop behind the window and appears immediately.

## grid/render_grid.rs → `Grid`

- Change: `Grid` gives each child its whole grid cell, with alignment expressed by child wrappers.
  Reason: framework — WinUI combines alignment and sizing on each element, while Reveal uses separate alignment wrappers and requires each render box to obey its parent’s size constraints.
  Affect: `Grid` constrains a child larger than its cell to that cell rather than letting it overflow.

## grid/layout.rs → `Grid` row and column layout

- Change: `Grid` row and column sizes are not rounded to physical pixels.
  Reason: framework — XAML rounds track sizes using the display scale, whereas Reveal’s layout keeps fractional logical-pixel measurements.
  Affect: `Grid` proportionally sized rows and columns can have edges between physical pixels.

## toggle_switch.rs → `ToggleSwitch`

- Change: `ToggleSwitch` moves its knob over 167 ms with fast-out-slow-in easing.
  Reason: os — the template delegates knob movement to Windows through `RepositionThemeAnimation`, leaving the duration and acceleration outside the checkout.
  Affect: `ToggleSwitch` slide duration and curve can differ from Windows.

- Change: `ToggleSwitch` waits for enough pointer movement to distinguish a drag from a tap before moving its knob.
  Reason: framework — Flutter recognizers first decide whether the press is a tap or a drag, while WinUI’s draggable knob starts tracking on press.
  Affect: `ToggleSwitch` keeps its knob still for the first 18 px of touch movement, and shorter movement remains a tap.

## radio_button.rs → `RadioButton`

- Change: `RadioButton` asks the caller to update the selected option instead of finding and unchecking other buttons in a named group.
  Reason: framework — Reveal’s controlled widgets receive their checked values from owner state, while WinUI can mutate other persistent RadioButton objects in the group.
  Affect: A `RadioButton` caller checks the chosen button and unchecks the others.

- Change: `RadioButton` keeps its selected dot’s outline when the pointer moves over it.
  Reason: framework — the port chooses one appearance for the current state rather than letting several WinUI animations overwrite the same color in sequence.
  Affect: Hovering a selected radio button does not briefly change its one-pixel dot outline.

## check_box.rs → `CheckBox`

- Change: `CheckBox` takes its parent’s allocated size without applying the style’s alignment.
  Reason: framework — WinUI combines alignment and sizing on each element, while Reveal uses separate alignment wrappers and requires each render box to obey its parent’s size constraints.
  Affect: In a stretched column, `CheckBox`’s hit area and focus ring span the column.

## repeat_button.rs → `RepeatButton`

- Change: `RepeatButton` uses the App clock with a 500 ms delay and 33 ms interval.
  Reason: os — the port schedules exact deadlines on Reveal’s application clock, whereas Windows rounds timer delivery to its system timer resolution.
  Affect: A held `RepeatButton` can repeat somewhat faster than on Windows.

## hyperlink_button.rs → `HyperlinkButton`

- Change: `HyperlinkButton` does not open a URL automatically.
  Reason: os — Windows opens the URL for the original control, but this host does not provide an equivalent URL-opening service.
  Affect: A `HyperlinkButton` caller opens the link from the click handler.

## slider.rs → `Slider`

- Change: `Slider` sizes its filled portion as a fraction of the available width.
  Reason: framework — the fill’s widget description is built before layout knows its width, so a proportional grid track expresses the size without waiting for another build.
  Affect: `Slider`’s fill can end on a fractional pixel where Windows rounds it.

- Change: `Slider` shows its value label using the kit’s tooltip and Flutter focus handling.
  Reason: framework — WinUI records whether mouse, keyboard or application code caused each focus change, while Reveal uses Flutter’s separate focus and focus-highlight handling.
  Affect: The slider needs an Overlay ancestor for its value label, whose placement and fade follow the tooltip behavior documented below.

- Change: `Slider` accepts a function returning text to format its value label.
  Reason: language — a Rust formatting callback replaces the object that converts bound values to text in XAML.
  Affect: `Slider` callers provide `thumb_tool_tip_value_converter` for custom text.

- Change: `Slider` vertical tooltip placement assumes a right-handed user.
  Reason: os — this host does not expose Windows handedness settings.
  Affect: `Slider` vertical tooltips prefer the left side of the thumb.

## split_view.rs → `SplitView`

- Change: `SplitView` dismisses an overlay pane when Reveal recognizes a tap outside it.
  Reason: framework — the pane lives in the application’s overlay, where tap recognition competes with other gestures instead of receiving pointer events from a separate XAML popup.
  Affect: `SplitView` requires an Overlay ancestor, and outside dismissal waits for a recognized tap.

- Change: `SplitView` updates automatic pane width after the pane is measured.
  Reason: framework — Reveal cannot read the child’s measured width during widget build.
  Affect: `SplitView` initial layout and content-width changes can take an extra frame to settle.

- Change: `SplitView` reports opening and closing requested by application code after the new layout is ready.
  Reason: framework — Reveal forbids updating ancestors or showing overlays during child build.
  Affect: `SplitView` opening and explicit closing notifications arrive after layout, while tapping outside the pane still raises a cancelable event before closing.

- Change: `SplitView` moves focus into its pane’s FocusScope when opening and can restore the previous FocusNode when closing.
  Reason: framework — WinUI associates an input cause with each focus change, while Reveal uses Flutter’s focus nodes and separate focus-highlight handling.
  Affect: The focus ring follows Flutter’s highlight handling when focus enters or returns from the pane, rather than the cause attached to a WinUI focus event.

## tool_tip/mod.rs → `ToolTip` and `ToolTipService`

- Change: `ToolTipService` draws help text in the application’s overlay rather than a separate popup window.
  Reason: framework — Reveal composes overlays and pointer listeners as widgets rather than attaching a popup service to any existing control.
  Affect: `ToolTipService` callers wrap the owner beneath an Overlay, and the tooltip stays inside that window.

- Change: `ToolTipService` uses a 150 ms fade and fixed delays before showing a tooltip.
  Reason: os — this host does not report Windows theme timing or user hover settings.
  Affect: Hovering first shows a tooltip after 800 ms, or after 400 ms if one was recently shown, regardless of the user’s Windows hover settings.

## navigation_scroll_viewport.rs → NavigationView navigation scroll presentation

- Change: `NavigationScrollViewport` uses WinUI’s scrollbar appearance with Flutter’s scrollbar input handling.
  Reason: framework — the port reuses Flutter’s existing behavior for dragging the scroll handle and clicking the track around it.
  Affect: `NavigationScrollViewport` scrolls once by 80% of the viewport on a track press instead of repeatedly paging while the pointer is held down.

- Change: `NavigationScrollViewport` changes its scrollbar thickness instantly after the hover delay.
  Reason: framework — Flutter’s scrollbar accepts a thickness value but does not animate that value itself.
  Affect: The scroll handle jumps to its new width instead of widening or narrowing over 167 ms.

## tab_view.rs → `TabView`

- Change: `TabView` receives its tab list and selection from the application, with a unique identity for each tab that stays the same across rebuilds.
  Reason: framework — Reveal rebuilds item descriptions from owner state instead of mutating a collection on the control.
  Affect: Callers apply close, reorder and transfer requests and update the selected index.

- Change: `TabView` builds its headers in a lazy ListView, and each tab keeps its key and focus node across rebuilds.
  Reason: framework — Flutter’s lazy list creates widgets as they come into view instead of using WinUI’s system for creating item containers.
  Affect: Offscreen selection first scrolls to an estimated position and then adjusts to measured headers.

- Change: `TabView` keeps inactive pages’ State alive, but those pages do not paint or receive animation ticks.
  Reason: framework — removing a Flutter page would dispose its State, whereas WinUI can keep the page object alive separately from its displayed content.
  Affect: Inactive pages can still be laid out; Offstage hides them and TickerMode mutes their animation ticks, while timers and other application updates still run.

- Change: `TabView` chooses a valid selected tab immediately when its tab list changes and reports that choice after the frame.
  Reason: framework — a Flutter child cannot trigger an ancestor rebuild while the ancestor’s current build is updating that child.
  Affect: The replacement `TabView` page appears in the current frame, while the owner’s selection callback runs after layout.

## tab_view/drag.rs → TabView tab dragging and reordering

- Change: `TabView` uses native drag recognition, an in-window preview and scrolling near the strip edges.
  Reason: framework — Reveal’s drag source and target communicate through the application overlay rather than Windows drag-and-drop services.
  Affect: Drag thresholds and edge-scroll speed follow Reveal, and transfers stay within one overlay.

- Change: `TabView` moves neighboring headers immediately after the 200 ms reorder delay.
  Reason: os — Windows supplies the source reposition animation.
  Affect: The insertion gap opens without the Windows motion animation.

- Change: `TabView` does not reserve space in a destination strip while a tab from another strip is hovering over it.
  Reason: framework — Reveal reports drag-target entry and movement but does not insert a temporary tab into the destination layout.
  Affect: Cross-strip hover does not widen the strip or open an insertion gap.

## tab_view_item.rs → `TabViewItem`

- Change: `TabViewItem` includes its text and icon in the default drag preview but omits other custom header widgets.
  Reason: framework — Reveal cannot capture the displayed header as an image, so it builds a separate widget for the drag preview.
  Affect: Applications supply a custom drag-preview builder for other header content, using keys and focus nodes separate from the original header.

- Change: `TabViewItem` uses Flutter gesture recognition and focus nodes, together with the kit’s tooltips.
  Reason: framework — WinUI can direct all events from a captured pointer to a control, whereas Flutter decides which gesture recognizer handles the press.
  Affect: A drag can cancel a tab click, and clicking a tab leaves keyboard focus unchanged, as described in the shared control conventions.

## navigation_indicator_transition.rs → NavigationView selection-indicator animations

- Change: `NavigationIndicatorTransition` starts moving the line marking selection after the newly selected navigation item is laid out.
  Reason: framework — the animation needs the new item’s position and size, which are unavailable until Flutter layout finishes.
  Affect: The line stays at the previously selected item for up to one extra frame before moving.

- Change: `NavigationIndicatorTransition` speeds up and slows down using an explicit ease-in-out curve when selection moves between a parent item and a nested child.
  Reason: os — the source leaves that acceleration to Windows, so the port substitutes Reveal’s cubic curve with control points (0.42, 0) and (0.58, 1).
  Affect: `NavigationIndicatorTransition` can speed up and slow down differently from Windows when changing hierarchy levels.

## navigation_view.rs → `NavigationView`

- Change: `NavigationView` receives its items and selection from the application, with a unique identity for each item that stays the same across rebuilds.
  Reason: framework — Reveal rebuilds item descriptions from owner state instead of mutating a XAML item collection on the control.
  Affect: Applications update `selected_item` in `selection_changed` and use `item_invoked` for activation, including activating an already selected item; item ids must remain unique and reserve `NavigationView::SETTINGS_ITEM_ID` for the built-in Settings item.

- Change: `NavigationView` uses Flutter gestures and focus handling, and reports pane changes after layout.
  Reason: framework — WinUI routes input events through its control objects, while this port composes the gesture, focus and pane widgets described above.
  Affect: Starting a scroll can cancel an item click, and application code receives pane-change callbacks after the frame.

- Change: `NavigationView` accepts any widget for its search field and item badges.
  Reason: framework — Reveal composes ordinary child widgets, while WinUI requires `AutoSuggestBox` for search and `InfoBadge` for badges.
  Affect: Applications can use the kit's `InfoBadge` or a custom badge, and supply their own search editor until `AutoSuggestBox` is ported.

- Change: `NavigationView` decides which top-level items fit and which go into the overflow menu after measuring them.
  Reason: framework — the widget tree is built before item widths are measured, so moving items between the main row and overflow requires a subsequent build.
  Affect: After item text changes, moving items between the top row and its overflow menu can take an extra frame.

## navigation_view_item.rs → NavigationView item containers

- Change: `NavigationViewItem` keeps its key and stays mounted when hidden or moved between the side pane and top bar.
  Reason: framework — Flutter State belongs to a mounted element, so keeping the element preserves custom item state across those moves.
  Affect: `NavigationViewItem` preserves custom item state across presentation changes, and its collapsed children remain mounted but cannot paint or take focus.

- Change: `NavigationViewItem` uses a replacement font symbol for its arrow and rotates it over 250 ms.
  Reason: os — Windows supplies the original animated arrow, while the replacement font contains only a static arrow.
  Affect: Navigation arrows can differ from Windows in shape and in how they move while opening or closing a child list.

## progress_bar.rs → `ProgressBar`

- Change: `ProgressBar` jumps to the new filled length when its value changes and omits the fade when switching from moving segments to a known progress value.
  Reason: os — the WinUI template asks Windows to perform those movement and fade effects through `RepositionThemeAnimation` and `FadeInThemeAnimation`, but their durations and acceleration are not defined in the checkout.
  Affect: `ProgressBar` still animates moving segments and pause/error colors using the timings written in the template; only the Windows-supplied effects above are omitted.

## expander.rs → `Expander`

- Change: `Expander` asks the application to change whether its content is shown.
  Reason: framework — like a controlled Flutter widget, it receives a boolean and a callback rather than changing the supplied boolean itself.
  Affect: The application must handle the callback and rebuild with the new value before the section opens or closes.

- Change: `Expander` reports that its content is opening or closing through callbacks that run after layout.
  Reason: framework — changing the supplied boolean rebuilds the widget, and a callback that updates application state cannot run safely during that build.
  Affect: Application code receives the notification at the end of the frame rather than immediately when it changes the boolean.

- Change: `Expander` rotates its expand/collapse arrow over 250 ms.
  Reason: os — Windows supplies the original animated icon, while the replacement font supplies only a static arrow that Reveal rotates.
  Affect: The arrow points in the correct direction, but its shape and movement differ from Windows.

- Change: `Expander` keeps hidden content’s State alive, but ticker-driven animations stop receiving ticks.
  Reason: framework — removing a Flutter widget would dispose its State, so hiding it preserves the child state for the next expansion.
  Affect: Child widget values survive closing and reopening, and TickerMode mutes their animation ticks while hidden; timers and other application updates still run.

## info_bar.rs → `InfoBar`

- Change: `InfoBar` asks the application to update whether it is open.
  Reason: framework — like a controlled Flutter widget, it receives a boolean and a callback rather than changing the supplied boolean itself.
  Affect: Handle `is_open_changed` by storing its requested boolean and rebuilding, including a request for `true` when the closing callback cancels a close started by application code.

- Change: `InfoBar` reports opening and closing requested by application code after layout.
  Reason: framework — these events may update application state, so they wait until the current build has finished.
  Affect: The opening notification arrives at the end of the frame, and a closing notification can then cancel the close before the banner is hidden.

- Change: `InfoBar` reports completion of a close-button request after the application rebuilds it as closed.
  Reason: framework — the button can request a new value, but only the application can supply that value to the next build.
  Affect: The click and cancelable closing callbacks run immediately; an accepted close reports completion only after the application rebuilds with the open value set to false, while a canceled close immediately reports that the banner is open again.

- Change: `InfoBar` keeps child State alive while closed, but ticker-driven animations stop receiving ticks.
  Reason: framework — removing a Flutter widget would dispose its State, so hiding the banner preserves the child state for the next opening.
  Affect: Child widget values survive closing and reopening, and TickerMode mutes their animation ticks while hidden; timers and other application updates still run.

- Change: `InfoBar` draws its status icons with the bundled replacement font over a colored circle.
  Reason: os — the original matching foreground and background symbols belong to a font supplied with Windows.
  Affect: The replacement icon shapes differ from Windows, while the warning uses an exclamation mark inside a circle that matches its circular background.

- Change: `InfoBar` accepts a widget-building callback to customize its close button.
  Reason: language — a Rust callback receives the button’s current state and content in place of a XAML style declaration.
  Affect: Applications can change the button’s appearance while keeping its existing click and close behavior.

- Change: `InfoBar` uses a callback rather than a command object for close-button actions.
  Reason: framework — WinUI command objects also report whether an action is available, while the kit’s button callbacks only perform the action.
  Affect: Applications capture action arguments in their callback, but the button does not automatically disable itself when that action becomes unavailable.

## progress_ring.rs → `ProgressRing`

- Change: `ProgressRing` supports the built-in loading and percentage animations, but does not accept a custom animation.
  Reason: framework — WinUI plays replaceable animation objects through Windows Composition, while this port draws the two built-in animations directly with Reveal's canvas and animation clock.
  Affect: Applications can change the range, value, activity and colors, but cannot replace the animation through WinUI's `DeterminateSource` or `IndeterminateSource` properties.

## info_badge.rs → `InfoBadge`

- Change: `InfoBadgeStyle` selects the badge's background color but does not supply an icon or icon-specific padding.
  Reason: os — WinUI's named icon styles depend on symbols from the Windows icon font, which this kit replaces with the bundled Fluent System Icons font.
  Affect: Applications supply `icon_source` themselves and use 4 logical pixels of top padding, 2 at the bottom and none on either side when reproducing the attention or informational icon styles.

## flyout/mod.rs → `Flyout` and `FlyoutBase`

- Change: `Flyout` keeps its content mounted while hidden and displays it inside the application’s root Overlay.
  Reason: framework — Flutter preserves child State through mounted widgets and draws overlays inside the existing window, while WinUI retains detached content and can create separate popup windows.
  Affect: Reopening the same Flyout from another button preserves its child State, but the flyout cannot extend beyond the application window.

- Change: `Flyout` uses Flutter’s ModalBarrier to block input outside its content when ShowMode is Standard.
  Reason: framework — WinUI's host distinguishes pointer buttons during popup hit testing, while Flutter's native barrier blocks input behind it and recognizes a completed tap from any button.
  Affect: Outside dismissal occurs on release rather than press, and an outside right-click is consumed rather than reaching the control behind the flyout.

- Change: `Flyout` opens and closes without a visual transition.
  Reason: os — WinUI obtains popup motion and timing from Windows rather than declaring them in the template.
  Affect: The flyout’s visible container appears and disappears immediately.

- Change: `Flyout` accepts a widget builder to customize the visible container around its content.
  Reason: language — Rust supplies widget construction through callbacks rather than runtime XAML template resources.
  Affect: Applications provide `flyout_presenter_style` when replacing the default border, scrolling or content layout.

## flyout/shadow.rs → `Flyout` and `MenuFlyout` shadows

- Change: `Flyout` and `MenuFlyout` paint their shadows using Reveal's native BoxShadow blur with WinUI's default shadow colors, offsets and increases in elevation for each nested submenu.
  Reason: os — Windows Composition performs WinUI's blur, and its blur-radius conversion is not specified, so the port interprets the radii from WinUI's `DropShadowRecipe.h` using Flutter's native conversion.
  Affect: Popups have a soft shadow that becomes stronger in dark theme and at deeper submenu levels, but its softness can differ from Windows; the shadow is clipped at the application window's edges.

## drop_down_button.rs → `DropDownButton`

- Change: `DropDownButton` uses a static Fluent chevron for the source animated icon.
  Reason: os — the Windows animated-chevron asset has no matching animation in the bundled Fluent font.
  Affect: The chevron changes color on hover and press but does not animate its shape.

## Deferred

- `ProgressRing` starts its arc at twelve o'clock, but that starting point remains unverified against Windows. Trigger: A Windows capture establishes where the original ellipse starts drawing its arc.

- Repeat buttons always activate on press rather than release. Trigger: A consumer needs release-time activation in the shared button handler.
- Arrow keys do not yet move between radio buttons in a group. Trigger: The group provides a list of buttons that can receive focus.
- Keeping the slider’s header widget mounted while its header is absent remains deferred. Trigger: A consumer needs that hidden header state to survive.
- ToggleSwitch state-color fades remain deferred. Trigger: The duration for each supported state change is identified from the source or measured on Windows.
- ToggleSwitch tap bounds still include the header. Trigger: A source review of hit areas using native gestures is available.
- ToggleSwitch knob animation timing remains unverified. Trigger: A recording from Windows is available.
- Collapsed Grid children are omitted rather than retained at zero size. Trigger: A consumer needs retained collapsed children.
- RepeatButton timer-rate verification remains deferred. Trigger: A Windows click-rate measurement is available.
- HyperlinkButton does not expose NavigateUri. Trigger: A host URL launcher is available.
- Slider default-value verification for StepFrequency, TickFrequency and SnapsTo remains deferred. Trigger: The Windows type table or a runtime measurement is available.
- Slider gamepad operation, including switching between navigating controls and adjusting the slider’s value, remains deferred. Trigger: A host supplies gamepad or remote-control input.
- The slider’s larger value step for accessibility actions remains deferred. Trigger: Accessibility support uses that step.
- SplitView system-back handling, element sounds and Xbox focus/dimming behavior remain deferred. Trigger: A host exposes those facilities.
- Template and Slider-key right-to-left mirroring remain deferred. Trigger: A consumer needs right-to-left layout, prompting a source comparison of each affected template and keyboard direction.
- CheckBox indeterminate sweep animation remains deferred. Trigger: A recording from Windows is available.
- `RatingControl` remains deferred because the bundled icon font supplies outlined stars but no filled stars for selected ratings. Trigger: A matching filled-star glyph is bundled.
- Additional TextBlock styles and dialogs remain deferred. Trigger: The next control port is selected.
- Mica wallpaper backdrops remain deferred. Trigger: A host material path supplies the wallpaper processing absent from the source.
- Text controls’ spell-check suggestion menus, touch command-bar presentation, input-scope-specific soft keyboards and OS credential/autofill integration remain deferred. Trigger: The corresponding native services or additional control ports are available.
- TabView tear-out, caption regions, window movement and cross-window drag transport remain deferred. Trigger: A host provides window management and native drag transport.
- Tab gamepad navigation and interaction, high-contrast colors and accessibility support remain deferred. Trigger: The corresponding input, theme and accessibility support is available.
- TabView insertion, removal and reorder animations remain deferred. Trigger: Windows timing is verified or a host animation service is available.
- Navigation integration with title-bar space, window borders, system sounds and page-change animations remains deferred. Trigger: The host exposes the corresponding window or animation support.
- Navigation by gamepad shoulder buttons, directional focus movement, accessibility and high-contrast colors remain deferred. Trigger: The corresponding input, accessibility and theme support is available.
- Standalone ScrollViewer and ScrollBar APIs remain deferred. Trigger: A consumer needs those public controls beyond NavigationView’s internal adapters.
