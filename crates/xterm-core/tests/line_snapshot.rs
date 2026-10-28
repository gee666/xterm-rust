use std::{any::Any, rc::Rc};
use xterm_core::buffer::{
    constants::{bg_flags, content},
    BufferLine, CellData,
};

#[test]
fn snapshot_keeps_real_words_stale_maps_cache_and_shared_payload() {
    let payload: Rc<dyn Any> = Rc::new(71u32);
    let mut cell = CellData {
        content: (1 << content::WIDTH_SHIFT) | content::IS_COMBINED_MASK,
        combined_data: vec![65, 0x301],
        ..CellData::default()
    };
    cell.bg = bg_flags::HAS_EXTENDED;
    cell.extended.set_payload(Some(payload.clone()));
    let mut line = BufferLine::new(2, Some(&cell), true);
    let text = line.translate_to_string(false, None, None, None);
    let mut snapshot = line.snapshot();
    assert_eq!(snapshot.length, 2);
    assert!(snapshot.is_wrapped);
    assert_eq!(snapshot.cache, Some((text, false)));
    assert_eq!(snapshot.combined.get(&0), Some(&vec![65, 0x301]));
    assert!(Rc::ptr_eq(
        &snapshot.extended[&0].payload().unwrap(),
        &payload
    ));
    snapshot.words[0][0] = 0;
    assert_ne!(line.snapshot().words[0][0], 0);
    let plain = CellData {
        content: (1 << content::WIDTH_SHIFT) | 66,
        ..CellData::default()
    };
    line.set_cell(0, &plain);
    let updated = line.snapshot();
    assert_eq!(updated.words[0][0], plain.content);
    assert_eq!(updated.combined.get(&0), Some(&vec![65, 0x301]));
    assert!(updated.extended.contains_key(&0));
    assert!(updated.cache.is_none());
}
