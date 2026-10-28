// Copyright (c) 2026 The xterm.js authors. MIT license.
// Recording hosts verify protocol order only; they never claim to render pixels.
use super::kitty::{command, decoder, number, optional};
use serde_json::{json, Value};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use xterm_addons::image::encoded::MemorySourceFactory;
use xterm_addons::image::kitty::handler::*;
use xterm_addons::image::kitty::storage::*;
use xterm_addons::image::storage::{AddImageOptions, ImageResource, StorageError};
use xterm_core::parser::handler::HandlerResult;

type Events = Rc<RefCell<Vec<Value>>>;
#[derive(Debug)]
struct FixtureError(&'static str);
impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for FixtureError {}

#[derive(Default)]
struct Placements {
    callback: RefCell<Option<DeletedCallback>>,
}
impl KittyPlacementStorage for Placements {
    fn add_image(&self, _: ImageResource, _: AddImageOptions) -> Result<u64, StorageError> {
        Err(StorageError::host(FixtureError(
            "display storage is a separate oracle",
        )))
    }
    fn delete_image(&self, id: u64) -> Result<(), StorageError> {
        let callback = self.callback.borrow().clone();
        if let Some(callback) = callback {
            callback(id)?;
        }
        Ok(())
    }
    fn on_deleted(&self) -> Option<DeletedCallback> {
        self.callback.borrow().clone()
    }
    fn set_on_deleted(&self, callback: Option<DeletedCallback>) {
        *self.callback.borrow_mut() = callback;
    }
}
struct Job {
    ready: Rc<Cell<Option<bool>>>,
    command: CommandRef,
}
struct Display {
    ready: Rc<Cell<Option<bool>>>,
    request: Option<KittyDisplayRequest>,
}
impl Future for Display {
    type Output = Result<(), StorageError>;
    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let Some(rendered) = self.ready.get() else {
            return Poll::Pending;
        };
        self.request.take();
        Poll::Ready(if rendered {
            Ok(())
        } else {
            Err(StorageError::host(FixtureError("render failure")))
        })
    }
}
struct Host {
    events: Events,
    jobs: RefCell<VecDeque<Job>>,
    throwing: Cell<bool>,
}
impl KittyGraphicsHost for Host {
    fn send_response(&self, data: &[u16]) -> Result<(), StorageError> {
        self.events.borrow_mut().push(json!(["reply", data]));
        if self.throwing.get() {
            Err(StorageError::host(FixtureError("reply failure")))
        } else {
            Ok(())
        }
    }
    fn begin_display(&self, request: KittyDisplayRequest) -> DisplayFuture {
        self.events.borrow_mut().push(json!([
            "display",
            number(request.image.id.get()),
            command(&request.command.borrow())
        ]));
        let ready = Rc::new(Cell::new(None));
        self.jobs.borrow_mut().push_back(Job {
            ready: Rc::clone(&ready),
            command: Rc::clone(&request.command),
        });
        Box::pin(Display {
            ready,
            request: Some(request),
        })
    }
}
struct NoWake;
impl Wake for NoWake {
    fn wake(self: Arc<Self>) {}
}

pub struct Runtime {
    handler: KittyGraphicsHandler,
    host: Rc<Host>,
    storage: Rc<KittyImageStorage>,
    events: Events,
    promises: VecDeque<HandlerResult>,
}
impl Runtime {
    pub fn new(limit: f64) -> Self {
        let events = Events::default();
        let host = Rc::new(Host {
            events: Rc::clone(&events),
            jobs: RefCell::default(),
            throwing: Cell::new(false),
        });
        let storage = Rc::new(KittyImageStorage::new(Rc::new(Placements::default())));
        let handler = KittyGraphicsHandler::new(
            host.clone(),
            Rc::clone(&storage),
            Rc::new(MemorySourceFactory),
            KittyOptions {
                size_limit: limit,
                pixel_limit: 16_777_216.0,
            },
        );
        Self {
            handler,
            host,
            storage,
            events,
            promises: VecDeque::new(),
        }
    }
    pub fn step(&mut self, operation: &Value) {
        if let Err(error) = self.perform(operation) {
            let mut kind = "Error";
            let mut current: &(dyn Error + 'static) = &error;
            for _ in 0..16 {
                if matches!(
                    current.downcast_ref::<KittyProtocolError>(),
                    Some(KittyProtocolError::InvalidControlCodepoint(_))
                ) {
                    kind = "RangeError";
                    break;
                }
                let Some(next) = current.source() else {
                    break;
                };
                current = next;
            }
            self.events
                .borrow_mut()
                .push(json!(["error", kind, error.to_string()]));
        }
    }
    fn perform(&mut self, operation: &Value) -> Result<(), StorageError> {
        match operation[0].as_str().unwrap() {
            "s" => self.handler.start()?,
            "p" => {
                let input: Vec<_> = operation[1]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|code| u32::try_from(code.as_u64().unwrap()).unwrap())
                    .collect();
                self.handler.put(&input)?;
            }
            "e" => match self.handler.end(operation[1].as_bool().unwrap())? {
                HandlerResult::Ready(value) => {
                    self.events.borrow_mut().push(json!(["end", "sync", value]))
                }
                HandlerResult::Failed(error) => return Err(StorageError::host(error)),
                pending => {
                    self.promises.push_back(pending);
                    self.events.borrow_mut().push(json!(["end", "async"]));
                }
            },
            "r" => {
                let job = self.host.jobs.borrow_mut().pop_front();
                let result = self.promises.pop_front();
                if let Some(job) = job {
                    job.ready.set(Some(operation[1].as_bool().unwrap()));
                    let mut future = Box::pin(result.unwrap().resolve());
                    let waker = Waker::from(Arc::new(NoWake));
                    let resolved = match future.as_mut().poll(&mut Context::from_waker(&waker)) {
                        Poll::Ready(result) => result.map_err(StorageError::host)?,
                        Poll::Pending => panic!("controlled display did not resolve"),
                    };
                    self.events.borrow_mut().push(json!(["resolved", resolved]));
                } else {
                    assert!(result.is_none());
                    self.events.borrow_mut().push(json!(["resolve", "none"]));
                }
            }
            "x" => self.handler.reset()?,
            "d" => self.handler.dispose()?,
            "throw" => self.host.throwing.set(operation[1].as_bool().unwrap()),
            "pq" | "pt" | "pe" | "pr" => {
                let pending = self
                    .handler
                    .pending_transmissions()
                    .get(operation[1].as_f64().unwrap())?
                    .unwrap();
                match operation[0].as_str().unwrap() {
                    "pq" => pending.command.borrow().borrow_mut().quiet = operation[2].as_f64(),
                    "pt" => pending
                        .total_encoded_size
                        .set(operation[2].as_f64().unwrap()),
                    "pe" => pending.decode_error.set(operation[2].as_bool().unwrap()),
                    _ => pending.decoder.borrow().borrow_mut().release(),
                }
            }
            "jc" => {
                let command = Rc::clone(&self.host.jobs.borrow()[0].command);
                let mut command = command.borrow_mut();
                match operation[1].as_str().unwrap() {
                    "quiet" => command.quiet = operation[2].as_f64(),
                    "id" => command.id = operation[2].as_f64(),
                    other => panic!("unknown job field {other}"),
                }
            }
            other => panic!("unknown fixture operation {other}"),
        }
        Ok(())
    }
    pub fn snapshot(&self) -> Value {
        let state = self.handler.diagnostics().unwrap();
        let pending: Vec<_> = self
            .handler
            .pending_transmissions()
            .iter()
            .map(|entry| {
                let (key, value) = entry.unwrap();
                json!({"key":number(key),"command":command(&value.command.borrow().borrow()),
                "total":number(value.total_encoded_size.get()),"error":value.decode_error.get(),
                "decoder":decoder(decoder_snapshot(&value.decoder.borrow()).unwrap())})
            })
            .collect();
        let images: Vec<_> = self.handler.images().iter().map(|entry| {
            let (key, image) = entry.unwrap();
            let bytes = image.bytes.borrow();
            json!({"key":number(key),"id":number(image.id.get()),"width":number(bytes.width),"height":number(bytes.height),
                "format":number(bytes.format),"compression":bytes.compression,"mime":bytes.data.mime(),"data":bytes.data.as_memory().unwrap()})
        }).collect();
        let placements: Vec<_> = self
            .handler
            .placements()
            .iter()
            .map(|entry| {
                let (id, value) = entry.unwrap();
                json!([number(id), value])
            })
            .collect();
        json!({"aborted":state.aborted,"decodeError":state.decode_error,"inControl":state.in_control,"control":state.control,
            "parsed":state.parsed.as_ref().map_or(Value::Null,command),"limit":number(state.encoded_limit),"total":number(state.total_encoded),
            "last":optional(state.last_pending),"active":state.active_decoder.map_or(Value::Null,decoder),"pending":pending,"images":images,
            "placements":placements,"lastImageId":number(self.storage.last_image_id().unwrap()),"events":self.events.borrow().clone()})
    }
    pub fn finished(&self) -> bool {
        self.promises.is_empty() && self.host.jobs.borrow().is_empty()
    }
}
