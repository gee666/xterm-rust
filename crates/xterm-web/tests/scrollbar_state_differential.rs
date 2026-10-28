use xterm_web::scroll::scrollbar_state::ScrollbarState;

fn number(text: &str) -> f64 {
    if text == "NaN" {
        f64::NAN
    } else {
        f64::from_bits(u64::from_str_radix(text, 16).unwrap())
    }
}
fn bits(value: f64) -> String {
    if value.is_nan() {
        "NaN".into()
    } else {
        format!("{:016x}", value.to_bits())
    }
}

#[test]
fn source_geometry_mutations_and_clones_match_1024_snapshots() {
    let mut state = None;
    let mut count = 0;
    for (row, line) in include_str!("fixtures/scrollbar-state.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 6);
        let args: Vec<_> = if fields[1].is_empty() {
            Vec::new()
        } else {
            fields[1].split(',').map(number).collect()
        };
        let mut returned = "-";
        if fields[0] == "new" {
            state = Some(ScrollbarState::new(
                args[0], args[1], args[2], args[3], args[4], args[5],
            ));
        } else {
            let current = state.as_mut().unwrap();
            match fields[0] {
                "setVisibleSize" => {
                    returned = if current.set_visible_size(args[0]) {
                        "true"
                    } else {
                        "false"
                    }
                }
                "setScrollSize" => {
                    returned = if current.set_scroll_size(args[0]) {
                        "true"
                    } else {
                        "false"
                    }
                }
                "setScrollPosition" => {
                    returned = if current.set_scroll_position(args[0]) {
                        "true"
                    } else {
                        "false"
                    }
                }
                "setScrollbarSize" => current.set_scrollbar_size(args[0]),
                "setArrowSize" => current.set_arrow_size(args[0]),
                "setOppositeScrollbarSize" => current.set_opposite_scrollbar_size(args[0]),
                "clone" => *current = current.clone(),
                other => panic!("unexpected operation {other}"),
            }
        }
        let current = state.as_ref().unwrap();
        let probe = number(fields[2]);
        let actual = [
            current.get_arrow_size(),
            current.get_scroll_position(),
            current.get_rectangle_large_size(),
            current.get_rectangle_small_size(),
            current.get_slider_size(),
            current.get_slider_position(),
            current.get_desired_scroll_position_from_offset(probe),
            current.get_desired_scroll_position_from_offset_paged(probe),
            current.get_desired_scroll_position_from_delta(probe),
        ]
        .map(bits)
        .join(",");
        assert_eq!(returned, fields[3], "return row {row}");
        assert_eq!(current.is_needed(), fields[4] == "1", "needed row {row}");
        assert_eq!(actual, fields[5], "row {row} {} seed 0x5c7011", fields[0]);
        count += 1;
    }
    assert_eq!(count, 1024);
}
