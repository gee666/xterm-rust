use std::{cell::{Cell,RefCell},rc::Rc};
use xterm_core::{headless::{NativeWritePump,NativeWriteScheduler},input::write_buffer::{WriteAction,WriteBuffer,WriteData,WriteError}};
#[test]
fn callbacks_and_write_timers_share_fifo_without_inline_execution() {
    let pump=NativeWritePump::with_clock(||100.0);let log=Rc::new(RefCell::new(Vec::new()));
    let output=Rc::clone(&log);let first=WriteBuffer::new(pump.scheduler(),move |_,_,_|{output.borrow_mut().push("a");Ok(WriteAction::Complete)});
    first.write(WriteData::text("a"),None).unwrap();
    let output=Rc::clone(&log);pump.scheduler().request_callback(move ||{output.borrow_mut().push("callback");Ok(())}).unwrap();
    let output=Rc::clone(&log);let last=WriteBuffer::new(pump.scheduler(),move |_,_,_|{output.borrow_mut().push("c");Ok(WriteAction::Complete)});
    last.write(WriteData::text("c"),None).unwrap();assert!(log.borrow().is_empty());pump.poll().unwrap();
    assert_eq!(*log.borrow(),["a","callback","c"]);
}
#[test]
fn callback_error_keeps_original_type_and_leaves_later_work_owned() {
    let pump=NativeWritePump::with_clock(||100.0);let called=Rc::new(Cell::new(false));
    pump.scheduler().request_callback(||Err(WriteError::external(std::io::Error::new(std::io::ErrorKind::PermissionDenied,"callback denied")))).unwrap();
    let output=Rc::clone(&called);pump.scheduler().request_callback(move ||{output.set(true);Ok(())}).unwrap();
    let errors=pump.poll().unwrap_err();assert_eq!(errors[0].external_ref::<std::io::Error>().unwrap().kind(),std::io::ErrorKind::PermissionDenied);
    assert!(!called.get());assert_eq!(pump.pending_tasks(),1);pump.poll().unwrap();assert!(called.get());
}
struct CancelOnDrop {scheduler:NativeWriteScheduler,handle:u64}
impl Drop for CancelOnDrop {fn drop(&mut self){self.scheduler.cancel_timer(self.handle);}}
#[test]
fn cancelling_callback_drops_captures_outside_scheduler_borrow() {
    let pump=NativeWritePump::with_clock(||100.0);let scheduler=pump.scheduler();let called=Rc::new(Cell::new(false));
    let output=Rc::clone(&called);let later=scheduler.request_callback(move ||{output.set(true);Ok(())}).unwrap();
    let captured=CancelOnDrop{scheduler:scheduler.clone(),handle:later};
    let primary=scheduler.request_callback(move ||{drop(captured);Ok(())}).unwrap();
    scheduler.cancel_timer(primary);assert_eq!(pump.pending_tasks(),0);assert_eq!(pump.poll().unwrap(),0);assert!(!called.get());
}
