// Shared source-generated traces for the native and browser queue adapters.
pub struct Case {
    pub mode: String,
    pub action: String,
    pub trigger: usize,
    pub initial: Vec<String>,
    pub expected: Expected,
}
#[derive(Debug, PartialEq)]
pub struct Expected {
    pub trace: Vec<String>,
    pub error: Option<String>,
    pub remaining: usize,
    pub pending: bool,
}
pub fn cases() -> Vec<Case> {
    include_str!("../fixtures/task-queue-clock.tsv")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), 8);
            Case {
                mode: fields[0].into(),
                action: fields[1].into(),
                trigger: fields[2].parse().unwrap(),
                initial: fields[3].split(',').map(String::from).collect(),
                expected: Expected {
                    trace: fields[4].split('|').map(String::from).collect(),
                    error: (fields[5] != "none").then(|| fields[5].into()),
                    remaining: fields[6].parse().unwrap(),
                    pending: fields[7].parse().unwrap(),
                },
            }
        })
        .collect()
}
