use std::{collections::BTreeSet, rc::Rc};
use xterm_web::links::{Link, Position, ProviderReplies, ProviderRequest, Range, RangePosition};
fn links(value: &str) -> Option<Vec<Rc<Link>>> {
    if value == "-" {
        return None;
    }
    if value.is_empty() {
        return Some(vec![]);
    }
    Some(
        value
            .split(',')
            .map(|record| {
                let p: Vec<_> = record.split(':').collect();
                Rc::new(Link {
                    text: p[0].encode_utf16().collect(),
                    range: Range {
                        start: RangePosition {
                            x: p[1].parse().unwrap(),
                            y: p[2].parse().unwrap(),
                        },
                        end: RangePosition {
                            x: p[3].parse().unwrap(),
                            y: p[4].parse().unwrap(),
                        },
                    },
                })
            })
            .collect(),
    )
}
#[test]
fn original_priority_cache_and_overlap_seeded_scenarios() {
    let mut scenario = None;
    let mut replies = ProviderReplies::default();
    let mut request = ProviderRequest::new(Position { x: 0, y: 0 });
    let mut received = BTreeSet::new();
    for (step, line) in include_str!("fixtures/link-replies.tsv")
        .lines()
        .enumerate()
    {
        let p: Vec<_> = line.split('\t').collect();
        assert_eq!(p.len(), 10, "step{step}");
        let id: usize = p[0].parse().unwrap();
        let count = p[2].parse().unwrap();
        let cols = p[3].parse().unwrap();
        let position = Position {
            x: p[4].parse().unwrap(),
            y: p[5].parse().unwrap(),
        };
        if scenario != Some(id) {
            scenario = Some(id);
            replies = ProviderReplies::default();
            request = ProviderRequest::new(position);
            received.clear();
        }
        let selected = match p[1] {
            "reply" => {
                let index = p[6].parse().unwrap();
                received.insert(index);
                replies
                    .receive(index, links(p[7]), &request, count, cols)
                    .into_iter()
                    .collect::<Vec<_>>()
            }
            "cache" => replies.from_cache(&ProviderRequest::new(position), count, cols),
            _ => panic!("unknown fixture operation"),
        };
        let selected = selected
            .iter()
            .map(|link| String::from_utf16(&link.text).unwrap())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(selected, p[8], "selection step{step}: {line}");
        let cache = received
            .iter()
            .map(|index| {
                let names = replies
                    .links(*index)
                    .map(|links| {
                        links
                            .iter()
                            .map(|link| String::from_utf16(&link.text).unwrap())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .unwrap_or_else(|| "-".into());
                format!("{index}={names}")
            })
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(cache, p[9], "cache step{step}: {line}");
    }
}
