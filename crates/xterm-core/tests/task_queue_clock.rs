use std::{cell::RefCell, rc::Rc};
use xterm_core::task_queue::{CallbackKind, Scheduler, TaskQueue, TaskQueueError, WeakTaskQueue};
#[path = "support/task_queue_clock.rs"]
mod fixture;
use fixture::{cases, Expected};
#[derive(Default)]
struct HostState {
    trace: Vec<String>,
    next: usize,
    clock: usize,
    queue: Option<WeakTaskQueue<Host>>,
}
struct Host {
    state: Rc<RefCell<HostState>>,
    action: String,
    trigger: usize,
}
impl Scheduler for Host {
    type Handle = usize;
    type Error = &'static str;
    fn request(&self, _: CallbackKind) -> Result<usize, Self::Error> {
        let mut state = self.state.borrow_mut();
        state.next += 1;
        let id = state.next;
        state.trace.push(format!("request:{id}"));
        Ok(id)
    }
    fn cancel(&self, id: usize) {
        self.state.borrow_mut().trace.push(format!("cancel:{id}"));
    }
    fn now(&self) -> f64 {
        let (clock, queue) = {
            let mut state = self.state.borrow_mut();
            state.clock += 1;
            let clock = state.clock;
            state.trace.push(format!("now:{clock}"));
            (clock, state.queue.as_ref().unwrap().upgrade().unwrap())
        };
        if clock == self.trigger {
            if self.action == "flush" {
                queue.flush().unwrap();
            } else {
                queue.clear();
                if self.action != "clear" {
                    let state = self.state.clone();
                    let fail = self.action == "replace-error";
                    queue
                        .enqueue_try(move |_| {
                            state.borrow_mut().trace.push("task:new".into());
                            if fail {
                                Err("sentinel")
                            } else {
                                Ok(false)
                            }
                        })
                        .unwrap();
                }
            }
        }
        0.0
    }
    fn deadline_exceeded(&self, _: f64) -> Result<(), Self::Error> {
        panic!("unexpected deadline warning");
    }
}
#[test]
fn clock_reentry_matches_unchanged_source_traces() {
    let cases = cases();
    assert_eq!(cases.len(), 32);
    for case in cases {
        let state = Rc::new(RefCell::new(HostState::default()));
        let host = Host {
            state: state.clone(),
            action: case.action.clone(),
            trigger: case.trigger,
        };
        let queue = TaskQueue::idle(host, case.mode == "idle");
        state.borrow_mut().queue = Some(queue.downgrade());
        for name in &case.initial {
            let state = state.clone();
            let name = name.clone();
            queue
                .enqueue(move |_| {
                    state.borrow_mut().trace.push(format!("task:{name}"));
                    false
                })
                .unwrap();
        }
        let result = if case.mode == "idle" {
            queue.process(|| {
                state.borrow_mut().trace.push("deadline".into());
                100.0
            })
        } else {
            queue.process_priority()
        };
        let error = result.err().map(|error| match error {
            TaskQueueError::MissingTask => "missing-task".to_string(),
            TaskQueueError::Callback(error) => {
                assert_eq!(error, "sentinel");
                "callback".to_string()
            }
        });
        let actual = Expected {
            trace: state.borrow().trace.clone(),
            error,
            remaining: queue.remaining_tasks(),
            pending: queue.has_pending_callback(),
        };
        assert_eq!(
            actual, case.expected,
            "{} {} clock{} {:?}",
            case.mode, case.action, case.trigger, case.initial
        );
        // Failed task dispatch retains its replacement and scheduled token.
        // Clearing after comparison also exercises recovery from MissingTask.
        queue.clear();
        queue.enqueue(|_| false).unwrap();
        queue.flush().unwrap();
    }
}
