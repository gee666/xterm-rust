//! Numeric source-oracle runner; no DOM or upstream engine executes here.
use serde_json::{json, Value};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    error::Error,
    io::{self, BufRead},
    rc::Rc,
};
use xterm_core::event::EmitError;
use xterm_web::{
    links::debounce::FrameScheduler,
    scroll::{
        scrollable::{ScrollError, Scrollable},
        state::{DimensionUpdate, PositionUpdate, ScrollState},
    },
};

#[derive(Default)]
struct Frames {
    next: Cell<u32>,
    queue: RefCell<VecDeque<u32>>,
    requested: RefCell<Vec<u32>>,
    canceled: RefCell<Vec<u32>>,
    fail_request: Cell<bool>,
    fail_cancel: Cell<bool>,
    fail_event: Cell<bool>,
    cancel_event: Cell<bool>,
}
struct Scheduler(Rc<Frames>);
impl FrameScheduler for Scheduler {
    type Handle = u32;
    type Error = &'static str;
    fn request_frame(&mut self) -> Result<u32, &'static str> {
        if self.0.fail_request.get() {
            return Err("request");
        }
        let id = self.0.next.get() + 1;
        self.0.next.set(id);
        self.0.queue.borrow_mut().push_back(id);
        self.0.requested.borrow_mut().push(id);
        Ok(id)
    }
    fn cancel_frame(&mut self, id: u32) -> Result<(), &'static str> {
        if self.0.fail_cancel.get() {
            return Err("cancel");
        }
        self.0.queue.borrow_mut().retain(|current| *current != id);
        self.0.canceled.borrow_mut().push(id);
        Ok(())
    }
    fn cleanup_error(&mut self, error: &'static str) {
        eprintln!("cleanup: {error}");
    }
}
fn number(value: &Value) -> Result<f64, Box<dyn Error>> {
    if let Some(value) = value.as_f64() {
        return Ok(value);
    }
    Ok(value.as_str().ok_or("expected numeric string")?.parse()?)
}
fn optional(value: &Value) -> Result<Option<f64>, Box<dyn Error>> {
    if value.is_null() {
        Ok(None)
    } else {
        number(value).map(Some)
    }
}
fn encoded(value: f64) -> String {
    if value.is_nan() {
        "NaN".into()
    } else if value == f64::INFINITY {
        "Infinity".into()
    } else if value == f64::NEG_INFINITY {
        "-Infinity".into()
    } else if value == 0.0 && value.is_sign_negative() {
        "-0".into()
    } else {
        value.to_string()
    }
}
fn state(value: ScrollState) -> Vec<String> {
    [
        value.width,
        value.scroll_width,
        value.left,
        value.height,
        value.scroll_height,
        value.top,
        value.raw_left,
        value.raw_top,
    ]
    .map(encoded)
    .to_vec()
}
struct World {
    model: Scrollable<Scheduler>,
    now: Rc<Cell<f64>>,
    frames: Rc<Frames>,
    events: Rc<RefCell<Vec<Value>>>,
}
impl World {
    fn new(integer: bool, duration: f64, now: f64) -> Self {
        let clock = Rc::new(Cell::new(now));
        let frames = Rc::new(Frames::default());
        let events = Rc::new(RefCell::new(Vec::new()));
        let read = clock.clone();
        let model = Scrollable::new(Scheduler(frames.clone()), integer, duration, move || {
            read.get()
        });
        let weak = model.downgrade();
        let output = events.clone();
        let flags = frames.clone();
        model.on_scroll().try_subscribe(move |event| {
            let previous = event.previous;
            let current = event.current;
            let changed = event.changed;
            output.borrow_mut().push(json!([
                ([
                    previous.width,
                    previous.scroll_width,
                    previous.left,
                    previous.height,
                    previous.scroll_height,
                    previous.top
                ]
                .map(encoded)),
                ([
                    current.width,
                    current.scroll_width,
                    current.left,
                    current.height,
                    current.scroll_height,
                    current.top
                ]
                .map(encoded)),
                event.in_smooth_scrolling,
                [
                    changed.width,
                    changed.scroll_width,
                    changed.left,
                    changed.height,
                    changed.scroll_height,
                    changed.top
                ]
            ]));
            if flags.fail_event.get() {
                return Err(EmitError::callback(io::Error::other("event")));
            }
            if flags.cancel_event.get() && event.in_smooth_scrolling {
                weak.upgrade()
                    .ok_or(EmitError::ListenerArrayCleared)?
                    .set_scroll_position_now(PositionUpdate {
                        top: Some(8.0),
                        left: None,
                    })
                    .map_err(|error| EmitError::callback(io::Error::other(error.to_string())))?;
            }
            Ok(())
        });
        Self {
            model,
            now: clock,
            frames,
            events,
        }
    }
    fn snapshot(&self, error: Option<&str>) -> Value {
        let future = self.model.future_scroll_position();
        json!([
            state(self.model.scroll_state()),
            [encoded(future.left), encoded(future.top)],
            self.model.has_pending_scroll_animation(),
            self.frames
                .queue
                .borrow()
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            *self.frames.requested.borrow(),
            *self.frames.canceled.borrow(),
            *self.events.borrow(),
            error
        ])
    }
}
fn action(world: &World, row: &[Value]) -> Result<(), Box<dyn Error>> {
    let result = match row[0].as_str().ok_or("missing command")? {
        "dims" => world.model.set_scroll_dimensions(
            DimensionUpdate {
                width: optional(&row[1])?,
                scroll_width: optional(&row[2])?,
                height: optional(&row[3])?,
                scroll_height: optional(&row[4])?,
            },
            row[5].as_bool().ok_or("raw flag")?,
        ),
        "now" => world.model.set_scroll_position_now(PositionUpdate {
            left: optional(&row[1])?,
            top: optional(&row[2])?,
        }),
        "smooth" => world.model.set_scroll_position_smooth(
            PositionUpdate {
                left: optional(&row[1])?,
                top: optional(&row[2])?,
            },
            row[3].as_bool().ok_or("reuse flag")?,
        ),
        "frame" => {
            if world.frames.queue.borrow_mut().pop_front().is_none() {
                return Err("frame".into());
            }
            world.model.frame()
        }
        "time" => {
            world.now.set(number(&row[1])?);
            return Ok(());
        }
        "duration" => {
            world.model.set_smooth_scroll_duration(number(&row[1])?);
            return Ok(());
        }
        "requestFail" => {
            world
                .frames
                .fail_request
                .set(row[1].as_bool().ok_or("request flag")?);
            return Ok(());
        }
        "cancelFail" => {
            world
                .frames
                .fail_cancel
                .set(row[1].as_bool().ok_or("cancel flag")?);
            return Ok(());
        }
        "eventFail" => {
            world
                .frames
                .fail_event
                .set(row[1].as_bool().ok_or("event flag")?);
            return Ok(());
        }
        "eventCancel" => {
            world
                .frames
                .cancel_event
                .set(row[1].as_bool().ok_or("cancel event flag")?);
            return Ok(());
        }
        "dispose" => world.model.dispose(),
        _ => return Err("unknown command".into()),
    };
    result.map_err(|error| match error {
        ScrollError::Scheduler(error) => error.into(),
        ScrollError::Dispatch(error) => error.to_string().into(),
    })
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut world = World::new(false, 0.0, 1000.0);
    for line in io::stdin().lock().lines() {
        let row: Value = serde_json::from_str(&line?)?;
        let row = row.as_array().ok_or("command array required")?;
        let error = if row[0] == "new" {
            world.frames.fail_cancel.set(false);
            world
                .model
                .dispose()
                .map_err(|error| io::Error::other(error.to_string()))?;
            world = World::new(
                row[1].as_bool().ok_or("integer flag")?,
                number(&row[2])?,
                number(&row[3])?,
            );
            None
        } else {
            action(&world, row).err().map(|error| error.to_string())
        };
        println!("{}", world.snapshot(error.as_deref()));
    }
    Ok(())
}
