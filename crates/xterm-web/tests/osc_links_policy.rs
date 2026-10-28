use std::cell::RefCell;
use xterm_core::buffer::{AttributeData, BufferLine};
use xterm_web::links::{try_osc_links_with_policy, OscBuffer};
fn row() -> BufferLine {
    let mut line = BufferLine::new(5, None, false);
    for (x, id) in [1, 1, 2, 2, 0].into_iter().enumerate() {
        let mut attr = AttributeData::default();
        attr.extended.set_url_id(id);
        attr.update_extended();
        line.set_cell_from_codepoint(x, u32::from('a'), 1, &attr);
    }
    line
}
struct Live {
    original: BufferLine,
    current: RefCell<BufferLine>,
}
impl OscBuffer for Live {
    type Error = &'static str;
    fn with_line<R>(
        &self,
        index: usize,
        read: impl FnOnce(&BufferLine) -> R,
    ) -> Result<Option<R>, Self::Error> {
        Ok((index == 0).then(|| read(&self.current.borrow())))
    }
    fn with_scan_line<R>(
        &self,
        index: usize,
        read: impl FnOnce(&BufferLine) -> R,
    ) -> Result<Option<R>, Self::Error> {
        Ok((index == 0).then(|| read(&self.original)))
    }
}
#[test]
fn policy_getter_runs_for_each_finished_nonempty_uri_after_lookup() {
    let source = Live {
        original: row(),
        current: RefCell::new(row()),
    };
    let log = RefCell::new(Vec::new());
    let result = try_osc_links_with_policy(
        &source,
        1,
        |id| {
            log.borrow_mut().push(format!("uri{id}"));
            Ok(Some(vec![id as u16]))
        },
        |_| {
            log.borrow_mut().push("policy".into());
            Ok(true)
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(*log.borrow(), vec!["uri1", "policy", "uri2", "policy"]);
}
#[test]
fn empty_uri_suppresses_policy_and_getter_failure_stops_scanning() {
    let source = Live {
        original: row(),
        current: RefCell::new(row()),
    };
    let lookups = RefCell::new(Vec::new());
    let result = try_osc_links_with_policy(
        &source,
        1,
        |id| {
            lookups.borrow_mut().push(id);
            Ok(Some(if id == 1 { vec![] } else { vec![b'x' as u16] }))
        },
        |_| Err("getter"),
    );
    assert_eq!(result, Err("getter"));
    assert_eq!(*lookups.borrow(), vec![1, 2]);
}
#[test]
fn outer_scan_retains_original_row_when_policy_replaces_buffer_row() {
    let source = Live {
        original: row(),
        current: RefCell::new(row()),
    };
    let result = try_osc_links_with_policy(
        &source,
        1,
        |id| Ok(Some(vec![id as u16])),
        |_| {
            *source.current.borrow_mut() = BufferLine::new(5, None, false);
            Ok(true)
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].text, vec![1]);
    assert_eq!(result[1].text, vec![2]);
}
