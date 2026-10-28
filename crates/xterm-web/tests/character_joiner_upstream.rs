// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use serde_json::Value;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use xterm_web::character_joiner::CharacterJoinerService;
#[path = "support/cells.rs"]
mod cells;
fn ranges(value: &Value) -> Vec<[usize; 2]> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|range| [cells::number(&range[0]), cells::number(&range[1])])
        .collect()
}
#[test]
fn replay_every_original_character_joiner_assertion() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/character-joiner.json")).unwrap();
    let cases = fixture["calls"].as_array().unwrap();
    assert_eq!(cases.len(), 27);
    for (index, case) in cases.iter().enumerate() {
        let service =
            CharacterJoinerService::new(|error| panic!("unexpected source joiner error: {error}"));
        let mut all_queues = Vec::new();
        for id in case["handlers"].as_array().unwrap() {
            let queue: VecDeque<_> = case["handlerCalls"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|entry| entry["id"] == *id)
                .cloned()
                .collect();
            let queue = Rc::new(RefCell::new(queue));
            all_queues.push(Rc::clone(&queue));
            service.register(Rc::new(move |text| {
                let call = queue
                    .borrow_mut()
                    .pop_front()
                    .expect("unexpected handler invocation");
                let expected: Vec<u16> = call["text"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| cells::number(value) as u16)
                    .collect();
                assert_eq!(text, expected, "source callback text");
                Ok(ranges(&call["ranges"]))
            }));
        }
        let line = case["cells"]
            .as_array()
            .map(|raw| Rc::new(RefCell::new(cells::line(raw))));
        assert_eq!(
            service.get_joined_characters(line),
            ranges(&case["expected"]),
            "original captured call {index}"
        );
        for queue in all_queues {
            assert!(
                queue.borrow().is_empty(),
                "source handler calls omitted in case {index}"
            );
        }
    }
}
