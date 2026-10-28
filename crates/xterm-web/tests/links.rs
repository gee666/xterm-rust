use std::rc::Rc;
use xterm_core::{
    buffer::{BufferLine, CellData, CharData},
    lifecycle::Disposable,
};
use xterm_web::links::{osc_links, Link, LinkProviderService, Range, RangePosition};

fn line(text: &str, ids: &[u32], wrapped: bool) -> BufferLine {
    let mut line = BufferLine::new(text.len(), None, wrapped);
    for (x, cp) in text.chars().enumerate() {
        let mut cell = CellData::from_char_data(&CharData {
            attr: 0,
            chars: vec![cp as u16],
            width: 1,
            code: Some(cp as u32),
        });
        cell.extended.set_url_id(u64::from(ids[x]));
        cell.update_extended();
        line.set_cell(x, &cell);
    }
    line
}
fn links(lines: &[BufferLine], y: usize) -> Vec<Link> {
    osc_links(
        lines,
        y,
        |id| Some(format!("https://example.com/{id}").encode_utf16().collect()),
        false,
        |_| true,
    )
    .unwrap()
}
fn range(x1: usize, y1: usize, x2: usize, y2: usize) -> Range {
    Range {
        start: RangePosition {
            x: x1 as u64,
            y: y1 as u64,
        },
        end: RangePosition {
            x: x2 as u64,
            y: y2 as u64,
        },
    }
}

#[test]
fn upstream_expands_backward() {
    let lines = [
        line("aabbb", &[0, 0, 1, 1, 1], false),
        line("ccccx", &[1, 1, 1, 1, 0], true),
    ];
    let found = links(&lines, 2);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].range, range(3, 1, 4, 2));
}
#[test]
fn upstream_expands_forward() {
    let lines = [
        line("aaaaa", &[1; 5], false),
        line("bbccc", &[1, 1, 0, 0, 0], true),
    ];
    let found = links(&lines, 1);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].range, range(1, 1, 2, 2));
}
#[test]
fn upstream_different_ids_do_not_merge() {
    let lines = [
        line("aaaaa", &[1; 5], false),
        line("bbbcc", &[2, 2, 2, 0, 0], true),
    ];
    let found = links(&lines, 1);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].range, range(1, 1, 5, 1));
}

fn assert_underline(range: Range) {
    let link = Link {
        text: "foo".encode_utf16().collect(),
        range,
    };
    let e = link.underline_event(100, 0);
    assert_eq!(e.x1, link.range.start.x as i64 - 1);
    assert_eq!(e.y1, link.range.start.y as i64 - 1);
    assert_eq!(e.x2, link.range.end.x);
    assert_eq!(e.y2, link.range.end.y as i64 - 1);
}
#[test]
fn upstream_show_underline() {
    assert_underline(range(5, 1, 7, 1));
}
#[test]
fn upstream_hide_underline() {
    assert_underline(range(5, 1, 7, 1));
}
#[test]
fn upstream_show_wrapped_underline() {
    assert_underline(range(2, 1, 4, 2));
}
#[test]
fn upstream_hide_wrapped_underline() {
    assert_underline(range(2, 1, 4, 2));
}

#[test]
fn provider_identity_duplicates_and_repeated_disposal() {
    let service = LinkProviderService::default();
    let p = Rc::new("same");
    let other = Rc::new("same");
    let handle = service.register(Rc::clone(&p));
    let _second = service.register(Rc::clone(&p));
    let _other = service.register(Rc::clone(&other));
    assert_eq!(service.snapshot().len(), 3);
    handle.dispose();
    assert_eq!(service.snapshot().len(), 2);
    assert!(Rc::ptr_eq(&service.snapshot()[0], &p));
    handle.dispose();
    assert_eq!(service.snapshot().len(), 1);
    assert!(Rc::ptr_eq(&service.snapshot()[0], &other));
    handle.dispose();
    assert_eq!(service.snapshot().len(), 1);
    service.dispose();
    assert!(service.snapshot().is_empty());
    let _new = service.register(p);
    assert_eq!(service.snapshot().len(), 1);
}

#[test]
fn missing_empty_and_protocol_policy() {
    assert!(osc_links(&[], 0, |_| None, false, |_| true).is_none());
    assert_eq!(links(&[BufferLine::new(5, None, false)], 1), vec![]);
    let rows = [line("aab", &[1, 1, 0], false)];
    assert!(osc_links(
        &rows,
        1,
        |_| Some("invalid".encode_utf16().collect()),
        false,
        |_| false
    )
    .unwrap()
    .is_empty());
    assert_eq!(
        osc_links(
            &rows,
            1,
            |_| Some("invalid".encode_utf16().collect()),
            true,
            |_| panic!("protocol parser must not run when bypassed")
        )
        .unwrap()
        .len(),
        1
    );
}

#[test]
fn matches_source_single_cell_eol_early_continue() {
    assert!(links(&[line("a", &[1], false)], 1).is_empty());
}

#[test]
fn generated_endpoint_beyond_u32_survives_wasm_pointer_width() {
    let range = Range {
        start: RangePosition {
            x: 1,
            y: u64::from(u32::MAX),
        },
        end: RangePosition {
            x: 1,
            y: 1_u64 << 32,
        },
    };
    assert!(range.contains(
        xterm_web::links::Position {
            x: 80,
            y: u32::MAX as usize
        },
        80
    ));
    let link = Link {
        text: vec![b'x' as u16],
        range,
    };
    let event = link.underline_event(80, (u32::MAX - 1) as usize);
    assert_eq!((event.y1, event.y2), (0, 1));
    assert_eq!(link.range.end.y, 1_u64 << 32);
}

#[test]
fn scroll_offset_can_place_underline_above_viewport() {
    let link = Link {
        text: "x".encode_utf16().collect(),
        range: range(2, 1, 4, 2),
    };
    let e = link.underline_event(100, 5);
    assert_eq!((e.x1, e.y1, e.x2, e.y2), (1, -5, 4, -4));
}
