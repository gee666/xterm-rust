use std::{cell::RefCell, rc::Rc};
use xterm_core::decoration::OverviewPosition;
use xterm_web::renderer::color_zone_store::{
    ColorZoneOptions, ColorZonePadding, ColorZoneRef, ColorZoneStore,
};
const POSITIONS: [Option<OverviewPosition>; 5] = [
    None,
    Some(OverviewPosition::Full),
    Some(OverviewPosition::Left),
    Some(OverviewPosition::Center),
    Some(OverviewPosition::Right),
];
fn identity(zone: &ColorZoneRef, all: &mut Vec<ColorZoneRef>) -> usize {
    if let Some(index) = all.iter().position(|old| Rc::ptr_eq(old, zone)) {
        index
    } else {
        let index = all.len();
        all.push(zone.clone());
        index
    }
}
fn snapshot(
    store: &mut ColorZoneStore,
    retained: &[ColorZoneRef],
    all: &mut Vec<ColorZoneRef>,
) -> String {
    let zones = store.zones().unwrap();
    let format = |zones: &[ColorZoneRef], all: &mut Vec<ColorZoneRef>| {
        let values = zones
            .iter()
            .map(|zone| {
                let id = identity(zone, all);
                let value = zone.borrow();
                let units = value
                    .color
                    .iter()
                    .map(|u| format!("{u:x}"))
                    .collect::<Vec<_>>()
                    .join(",");
                let position = POSITIONS.iter().position(|p| *p == value.position).unwrap();
                format!(
                    "{id}:{units}:{position}:{}:{}",
                    value.start_buffer_line, value.end_buffer_line
                )
            })
            .collect::<Vec<_>>()
            .join(";");
        if values.is_empty() {
            "-".to_owned()
        } else {
            values
        }
    };
    format!("{}|{}", format(&zones.borrow(), all), format(retained, all))
}
#[test]
fn all_2048_zone_operations_and_retained_identities_match_javascript() {
    let colors = [
        vec![114, 101, 100],
        vec![98, 108, 117, 101],
        vec![0xd800],
        vec![114, 101, 100],
    ];
    let mut store = ColorZoneStore::new();
    let mut retained = Vec::new();
    let mut all = Vec::new();
    let fixture = include_str!("fixtures/color-zone.tsv");
    assert_eq!(fixture.lines().count(), 2048);
    for (index, row) in fixture.lines().enumerate() {
        let fields = row.split('\t').collect::<Vec<_>>();
        let op: usize = fields[0].parse().unwrap();
        let line: f64 = fields[1].parse().unwrap();
        let color: usize = fields[2].parse().unwrap();
        let position: usize = fields[3].parse().unwrap();
        let padding: f64 = fields[4].parse().unwrap();
        let options = ColorZoneOptions {
            color: colors[color].clone(),
            position: POSITIONS[position],
        };
        match op {
            0 => store.add(line, Some(&options)).unwrap(),
            1 => {
                store.clear().unwrap();
                store.add(line, Some(&options)).unwrap();
            }
            2 => store.set_padding(Rc::new(RefCell::new(ColorZonePadding {
                full: Some(padding),
                left: Some(padding + 1.0),
                center: Some(padding + 2.0),
                right: Some(padding - 1.0),
            }))),
            3 => {
                let zones = store.zones().unwrap();
                if let Some(zone) = zones.borrow().first().cloned() {
                    if retained.len() == 16 {
                        retained.remove(0);
                    }
                    retained.push(zone);
                };
            }
            4 => store.add(line, None).unwrap(),
            _ => panic!("unexpected fixture operation"),
        }
        assert_eq!(
            snapshot(&mut store, &retained, &mut all),
            fields[5],
            "operation {index}"
        );
    }
}
