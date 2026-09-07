//! The gallery under a driver that behaves like the winit embedder: frames only when requested, timers only when the platform's `wake_at` deadline is reached, the app clock advanced only by frames and wakes.
#![feature(arbitrary_self_types)]
mod common;
use common::CaptureView;
use reveal_embedder::valo::Context;
use reveal_embedder::{
    EmbedderClient, FontSource, Frame, Offset, Platform, PointerChange, PointerData,
    PointerDataPacket, PointerDeviceKind, SystemFontSource, TargetPlatform, ViewId, ViewRef,
};
use reveal_rendering::RenderParagraph;
use reveal_shell::Shell;
use reveal_widgets::WidgetsBinding;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

struct LoopPlatform {
    view: Rc<CaptureView>,
    origin: Instant,
    clock: Cell<Duration>,
    frame_requested: Cell<bool>,
    deadline: Cell<Option<Instant>>,
}
impl Platform for LoopPlatform {
    fn target_platform(&self) -> TargetPlatform {
        TargetPlatform::MacOS
    }
    fn request_frame(&self) {
        self.frame_requested.set(true);
    }
    fn now(&self) -> Instant {
        self.origin + self.clock.get()
    }
    fn wake_at(&self, deadline: Instant) {
        self.deadline.set(Some(deadline));
    }
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

struct Driver {
    platform: Rc<LoopPlatform>,
    shell: Shell,
    pointer_id: i64,
    log: Vec<String>,
}
impl Driver {
    fn new() -> Driver {
        let (device, queue) = valo_harness::headless_device().expect("GPU required");
        let view = Rc::new(CaptureView {
            size: [900, 2000],
            renderer: RefCell::new(Context::new(device, queue)),
            pixels: RefCell::new(Vec::new()),
        });
        let platform = Rc::new(LoopPlatform {
            view,
            origin: Instant::now(),
            // Real time has passed by the time the embedder's first event arrives.
            clock: Cell::new(Duration::from_millis(1)),
            frame_requested: Cell::new(false),
            deadline: Cell::new(None),
        });
        let shell = Shell::new(platform.clone(), winui_gallery::run);
        let mut driver = Driver {
            platform,
            shell,
            pointer_id: 0,
            log: Vec::new(),
        };
        driver.run_until(Duration::from_millis(500));
        driver
    }

    /// The winit loop: a requested frame runs at the next 16 ms tick; a reached deadline wakes the client; otherwise time passes.
    fn run_until(&mut self, end: Duration) {
        while self.platform.clock.get() < end {
            let now = self.platform.clock.get();
            if self.platform.frame_requested.replace(false) {
                self.log.push(format!("{now:?} frame"));
                self.shell.frame(Frame { elapsed: now });
                // The display's next vsync.
                self.platform.clock.set(now + Duration::from_millis(16));
            } else if let Some(deadline) = self.platform.deadline.get()
                && deadline <= self.platform.now()
            {
                self.platform.deadline.set(None);
                self.log.push(format!("{now:?} wake"));
                self.shell.wake(now);
            } else {
                let next = self
                    .platform
                    .deadline
                    .get()
                    .map(|d| d.saturating_duration_since(self.platform.origin))
                    .filter(|d| *d > now)
                    .unwrap_or(end)
                    .min(end)
                    .min(now + Duration::from_millis(16));
                self.platform.clock.set(next);
            }
        }
    }

    fn mouse(&mut self, change: PointerChange, point: Offset, down: bool) {
        if change == PointerChange::Down {
            self.pointer_id += 1;
        }
        let buttons = if down { 1 } else { 0 };
        self.log
            .push(format!("{:?} {change:?}", self.platform.clock.get()));
        self.shell
            .pointer_data_packet(PointerDataPacket::new(vec![PointerData {
                change,
                kind: PointerDeviceKind::Mouse,
                time_stamp: self.platform.clock.get(),
                pointer_identifier: if change == PointerChange::Hover && !down {
                    0
                } else {
                    self.pointer_id
                },
                physical_x: point.dx(),
                physical_y: point.dy(),
                buttons,
                ..Default::default()
            }]));
    }

    fn find(&self, text: &str) -> Option<Offset> {
        let cell = self.shell.cell();
        let mut app = cell.borrow_mut();
        let root = WidgetsBinding::instance(&mut app)
            .root_element(&app)
            .unwrap();
        let mut all = vec![root];
        let mut i = 0;
        while i < all.len() {
            all.extend(all[i].children(&app));
            i += 1;
        }
        for element in all {
            if let Some(object) = element.render_object(&app)
                && let Some(paragraph) = object.downcast::<RenderParagraph>(&app)
                && paragraph.text(&app).to_plain_text(true, true) == text
            {
                let object = object.as_box().unwrap();
                let size = object.size(&app);
                return Some(
                    object.local_to_global(&app, Offset::ZERO, None)
                        + Offset::new(size.width() / 2.0, size.height() / 2.0),
                );
            }
        }
        None
    }

    fn status(&self, prefix: &str) -> String {
        let cell = self.shell.cell();
        let mut app = cell.borrow_mut();
        let root = WidgetsBinding::instance(&mut app)
            .root_element(&app)
            .unwrap();
        let mut all = vec![root];
        let mut i = 0;
        while i < all.len() {
            all.extend(all[i].children(&app));
            i += 1;
        }
        for element in all {
            if let Some(object) = element.render_object(&app)
                && let Some(paragraph) = object.downcast::<RenderParagraph>(&app)
            {
                let t = paragraph.text(&app).to_plain_text(true, true);
                if t.starts_with(prefix) {
                    return t;
                }
            }
        }
        String::new()
    }
}

#[test]
fn repeat_button_repeats_under_the_winit_loop() {
    let mut driver = Driver::new();
    let hold_me = driver.find("Hold me").expect("Hold me");
    driver.mouse(PointerChange::Add, hold_me - Offset::new(50.0, 50.0), false);
    driver.mouse(PointerChange::Hover, hold_me, false);
    // Idle long enough for the app clock to fall behind the platform's between frames.
    driver.run_until(Duration::from_millis(1000));
    driver.mouse(PointerChange::Down, hold_me, true);
    driver.run_until(Duration::from_millis(1400));
    // `Delay` is 500 ms from the press, on the platform's clock.
    assert_eq!(driver.status("RepeatButton:"), "RepeatButton: 1 clicks");
    driver.run_until(Duration::from_millis(1600));
    let after_delay = driver.status("RepeatButton:");
    driver.run_until(Duration::from_millis(2500));
    let held = driver.status("RepeatButton:");
    driver.mouse(PointerChange::Up, hold_me, false);
    driver.run_until(Duration::from_millis(3000));
    let released = driver.status("RepeatButton:");
    let clicks = |status: &str| {
        status
            .trim_start_matches("RepeatButton: ")
            .trim_end_matches(" clicks")
            .parse::<u32>()
            .unwrap()
    };
    assert!(
        clicks(&after_delay) >= 3,
        "{after_delay} at 1.6 s; log:\n{}",
        driver.log.join("\n")
    );
    // The delay at 1500 ms, then 1000 ms of repeats at 33 ms: 32, give or take a frame.
    assert!(
        (30..=34).contains(&clicks(&held)),
        "{held} at 2.5 s; log:\n{}",
        driver.log.join("\n")
    );
    assert_eq!(released, held, "no clicks after the release");
}
