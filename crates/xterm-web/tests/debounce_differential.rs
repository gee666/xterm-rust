use std::{
    cell::{Cell, RefCell},
    convert::Infallible,
    rc::Rc,
};
use xterm_web::links::{
    debounce::{FrameScheduler, RenderDebouncer, RenderRange},
    time_debounce::{TimeBasedDebouncer, TimerScheduler},
};
fn number(value: f64) -> String {
    if value.is_nan() {
        "NaN".into()
    } else {
        value.to_string()
    }
}
#[derive(Clone, Default)]
struct Clock {
    now: Rc<Cell<f64>>,
    next: Rc<Cell<u32>>,
    log: Rc<RefCell<Vec<String>>>,
}
impl Clock {
    fn request(&self, kind: &str) -> u32 {
        let id = self.next.get() + 1;
        self.next.set(id);
        self.log.borrow_mut().push(format!("{kind}:{id}"));
        id
    }
    fn cancel(&self, id: u32) {
        self.log.borrow_mut().push(format!("cancel:{id}"));
    }
    fn render(&self, r: RenderRange) {
        self.log
            .borrow_mut()
            .push(format!("render:{}:{}", number(r.start), number(r.end)));
    }
}
impl FrameScheduler for Clock {
    type Handle = u32;
    type Error = Infallible;
    fn request_frame(&mut self) -> Result<u32, Infallible> {
        Ok(self.request("frame"))
    }
    fn cancel_frame(&mut self, id: u32) -> Result<(), Infallible> {
        self.cancel(id);
        Ok(())
    }
    fn cleanup_error(&mut self, error: Infallible) {
        match error {}
    }
}
impl TimerScheduler for Clock {
    type Handle = u32;
    type Error = Infallible;
    fn now(&self) -> f64 {
        self.now.get()
    }
    fn request_timeout(&mut self, delay: f64) -> Result<u32, Infallible> {
        Ok(self.request(&format!("timeout:{}", number(delay))))
    }
    fn cancel_timeout(&mut self, id: u32) -> Result<(), Infallible> {
        self.cancel(id);
        Ok(())
    }
    fn cleanup_error(&mut self, error: Infallible) {
        match error {}
    }
}
fn optional(value: &str) -> Option<f64> {
    (value != "-").then(|| value.parse().unwrap())
}
#[test]
fn seeded_original_frame_and_time_debounce_trace() {
    let frame_clock = Clock::default();
    let time_clock = Clock::default();
    let output = frame_clock.clone();
    let frame = RenderDebouncer::new(frame_clock.clone(), move |_, range| output.render(range));
    let output = time_clock.clone();
    let time = TimeBasedDebouncer::new(time_clock.clone(), 1000.0, move |_, range| {
        output.render(range)
    });
    for (step, line) in include_str!("fixtures/debounce.tsv").lines().enumerate() {
        let f: Vec<_> = line.split('\t').collect();
        assert_eq!(f.len(), 8, "step{step}");
        let clock = if f[0] == "frame" {
            &frame_clock
        } else {
            &time_clock
        };
        clock.now.set(f[2].parse().unwrap());
        let (start, end, count) = (optional(f[3]), optional(f[4]), optional(f[5]));
        if f[0] == "frame" {
            match f[1] {
                "refresh" => frame.refresh(start, end, count.unwrap()).unwrap(),
                "callback" => {
                    let clock = clock.clone();
                    let tag = f[6].to_owned();
                    frame
                        .add_refresh_callback(move |_, stamp| {
                            clock
                                .log
                                .borrow_mut()
                                .push(format!("callback:{tag}:{}", number(stamp)))
                        })
                        .unwrap();
                }
                "dispose" => frame.dispose().unwrap(),
                "dispatch" => {
                    if frame.has_pending_frame() {
                        frame.on_frame().unwrap();
                    }
                }
                _ => panic!("unknown frame op"),
            }
        } else {
            match f[1] {
                "refresh" => time.refresh(start, end, count.unwrap()).unwrap(),
                "dispose" => time.dispose().unwrap(),
                "dispatch" => {
                    if time.has_pending_timeout() {
                        time.on_timeout().unwrap();
                    }
                }
                _ => panic!("unknown time op"),
            }
        }
        let actual = std::mem::take(&mut *clock.log.borrow_mut()).join(",");
        assert_eq!(actual, f[7], "step{step}: {line}");
    }
}
