//! A GPU-backed host for kit tests: mounts a widget tree in a real `App` on a headless valo device, pumps frames, finds labels, sends pointer events and can export what it rendered.
#![allow(dead_code)]
use reveal_embedder::valo::{Color, Context};
use reveal_embedder::{
    FontSource, Offset, Picture, Platform, PointerChange, PointerData, PointerDataPacket,
    PointerDeviceKind, SystemFontSource, TargetPlatform, View, ViewConstraints, ViewId,
    ViewMetrics, ViewRef,
};
use reveal_foundation::{App, AppCell};
use reveal_gestures::GestureBinding;
use reveal_rendering::RenderParagraph;
use reveal_scheduler::SchedulerBinding;
use reveal_widgets::{AnyElement, WidgetRef, WidgetsBinding};
use std::{cell::RefCell, rc::Rc, time::Duration};

pub struct CaptureView {
    pub size: [u32; 2],
    pub renderer: RefCell<Context>,
    pub pixels: RefCell<Vec<u8>>,
}
impl View for CaptureView {
    fn id(&self) -> ViewId {
        ViewId(0)
    }
    fn metrics(&self) -> ViewMetrics {
        ViewMetrics {
            physical_size: self.size.map(|v| v as f64),
            physical_constraints: ViewConstraints::tight(self.size[0] as f64, self.size[1] as f64),
            device_pixel_ratio: 1.0,
            ..Default::default()
        }
    }
    fn present(&self, picture: &Picture) {
        *self.pixels.borrow_mut() =
            self.renderer
                .borrow_mut()
                .render_to_rgba(picture, self.size, Some(Color::WHITE));
    }
}
struct Host {
    view: Rc<CaptureView>,
}
impl Platform for Host {
    fn target_platform(&self) -> TargetPlatform {
        TargetPlatform::MacOS
    }
    fn request_frame(&self) {}
    fn now(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
    fn wake_at(&self, _deadline: std::time::Instant) {}
    fn views(&self) -> Vec<ViewRef> {
        vec![self.view.clone()]
    }
    fn view(&self, id: ViewId) -> Option<ViewRef> {
        (id == ViewId(0)).then(|| self.view.clone() as ViewRef)
    }
    fn implicit_view(&self) -> Option<ViewRef> {
        Some(self.view.clone())
    }
    fn font_source(&self) -> Option<Box<dyn FontSource>> {
        Some(Box::new(SystemFontSource::platform()))
    }
}
pub struct Fixture {
    pub cell: Rc<AppCell>,
    pub view: Rc<CaptureView>,
    pub at: Duration,
}
impl Fixture {
    /// Mounts the whole gallery.
    pub fn new(size: [u32; 2]) -> Self {
        Self::with_root(size, winui_gallery::run)
    }

    /// Mounts the gallery directly on the feature under test.
    pub fn for_feature(size: [u32; 2], feature: winui_gallery::Feature) -> Self {
        Self::with_root(size, move |app| winui_gallery::run_feature(app, feature))
    }

    /// Switches pages through the visible navigation item.
    pub fn navigate(&mut self, feature: winui_gallery::Feature) {
        self.ensure_visible(feature.title());
        self.tap(feature.title());
        self.pump();
    }

    /// Mounts whatever `run` starts, for a test of one control.
    pub fn with_root(size: [u32; 2], run: impl FnOnce(&mut App)) -> Self {
        let (device, queue) = valo_harness::headless_device().expect("GPU required");
        let view = Rc::new(CaptureView {
            size,
            renderer: RefCell::new(Context::new(device, queue)),
            pixels: RefCell::new(Vec::new()),
        });
        let cell = AppCell::with_platform(Rc::new(Host { view: view.clone() }));
        {
            let mut app = cell.borrow_mut();
            reveal_painting::PaintingBinding::instance(&mut app).install_fonts(&mut app, |fonts| {
                fonts.add_source(SystemFontSource::platform());
            });
            run(&mut app);
        }
        cell.elapse(Duration::ZERO);
        let mut result = Self {
            cell,
            view,
            at: Duration::ZERO,
        };
        result.pump();
        result
    }
    pub fn pump(&mut self) {
        for _ in 0..10 {
            self.at += Duration::from_millis(20);
            SchedulerBinding::handle_begin_frame(&mut self.cell.borrow_mut(), Some(self.at));
            self.cell.checkpoint();
            SchedulerBinding::handle_draw_frame(&mut self.cell.borrow_mut());
            self.cell.checkpoint();
        }
    }
    pub fn elements(&self) -> Vec<AnyElement> {
        let mut app = self.cell.borrow_mut();
        let root = WidgetsBinding::instance(&mut app)
            .root_element(&app)
            .unwrap();
        let mut all = vec![root];
        let mut i = 0;
        while i < all.len() {
            all.extend(all[i].children(&app));
            i += 1;
        }
        all
    }
    /// Enumerates painted subtrees while excluding descendants of Offstage widgets.
    fn onstage_elements(&self) -> Vec<AnyElement> {
        let mut app = self.cell.borrow_mut();
        let root = WidgetsBinding::instance(&mut app)
            .root_element(&app)
            .unwrap();
        let mut elements = vec![root];
        let mut index = 0;
        while index < elements.len() {
            let element = elements[index];
            let offstage = reveal_widgets::downcast_widget::<reveal_widgets::Offstage>(
                element.widget(&app).as_ref(),
            )
            .is_some_and(|widget| widget.offstage);
            if !offstage {
                elements.extend(element.children(&app));
            }
            index += 1;
        }
        elements
    }

    pub fn find(&self, text: &str) -> Offset {
        let elements = self.onstage_elements();
        let app = self.cell.borrow();
        for element in elements {
            if let Some(object) = element.render_object(&app)
                && let Some(paragraph) = object.downcast::<RenderParagraph>(&app)
                && paragraph.text(&app).to_plain_text(true, true) == text
            {
                let object = object.as_box().unwrap();
                let size = object.size(&app);
                return object.local_to_global(
                    &app,
                    Offset::new(size.width() / 2.0, size.height() / 2.0),
                    None,
                );
            }
        }
        panic!("missing label {text}")
    }
    /// Brings an already mounted label into view before a test sends pointer input.
    pub fn ensure_visible(&mut self, text: &str) {
        let elements = self.onstage_elements();
        let mut app = self.cell.borrow_mut();
        let context = elements
            .into_iter()
            .find(|element| {
                element
                    .render_object(&app)
                    .and_then(|object| object.downcast::<RenderParagraph>(&app))
                    .is_some_and(|paragraph| paragraph.text(&app).to_plain_text(true, true) == text)
            })
            .unwrap_or_else(|| panic!("missing label {text}"));
        drop(reveal_widgets::Scrollable::ensure_visible(
            &mut app,
            context,
            0.5,
            Duration::ZERO,
            reveal_animation::Curves::linear(),
            reveal_widgets::ScrollPositionAlignmentPolicy::Explicit,
        ));
        drop(app);
        self.cell.checkpoint();
        self.pump();
    }

    pub fn send(&mut self, change: PointerChange, point: Offset) {
        let mut app = self.cell.borrow_mut();
        GestureBinding::instance(&mut app).handle_pointer_data_packet(
            &mut app,
            PointerDataPacket::new(vec![PointerData {
                change,
                kind: PointerDeviceKind::Touch,
                time_stamp: self.at,
                pointer_identifier: 1,
                physical_x: point.dx(),
                physical_y: point.dy(),
                ..Default::default()
            }]),
        );
        drop(app);
        self.cell.checkpoint();
    }
    /// A mouse event: hover moves have no buttons, a press and its moves carry the primary button.
    pub fn send_mouse(&mut self, change: PointerChange, point: Offset, buttons: i64) {
        let mut app = self.cell.borrow_mut();
        GestureBinding::instance(&mut app).handle_pointer_data_packet(
            &mut app,
            PointerDataPacket::new(vec![PointerData {
                change,
                kind: PointerDeviceKind::Mouse,
                time_stamp: self.at,
                pointer_identifier: 7,
                physical_x: point.dx(),
                physical_y: point.dy(),
                buttons,
                ..Default::default()
            }]),
        );
        drop(app);
        self.cell.checkpoint();
    }
    pub fn tap(&mut self, text: &str) {
        let p = self.find(text);
        self.send(PointerChange::Down, p);
        self.at += Duration::from_millis(20);
        self.send(PointerChange::Up, p);
        self.pump();
    }
    pub fn capture(&self, name: &str) {
        let pixels = self.view.pixels.borrow();
        assert_eq!(
            pixels.len(),
            (self.view.size[0] * self.view.size[1] * 4) as usize
        );
        assert!(pixels.as_chunks::<4>().0.iter().all(|p| p[3] == 255));
        if let Ok(dir) = std::env::var("WINUI_CAPTURE_DIR") {
            let dir = std::path::Path::new(&dir);
            std::fs::create_dir_all(dir).unwrap();
            valo_harness::write_png(&dir.join(format!("{name}.png")), self.view.size, &pixels);
        }
    }
}

/// `reveal_widgets::run_app` for a widget built once the app exists.
pub fn mount(root: impl FnOnce(&mut App) -> WidgetRef + 'static) -> impl FnOnce(&mut App) {
    move |app| {
        let widget = root(app);
        reveal_widgets::run_app(app, widget);
    }
}
