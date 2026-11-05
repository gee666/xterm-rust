// GPOS layout reads and Position.init follow the supplied opentype.js parser.
use super::{Context, FontError, Result};

type Scripts = Vec<([u8; 4], Option<Vec<u16>>)>;
type Features = Vec<([u8; 4], Option<Vec<u16>>)>;

impl<E, F: FnMut(super::FontWarning) -> std::result::Result<Option<Vec<u8>>, E>>
    Context<'_, '_, F>
{
    fn tag(&self, at: u64) -> std::result::Result<[u8; 4], FontError> {
        let mut tag = [0; 4];
        tag.copy_from_slice(
            self.reader()
                .bytes(usize::try_from(at).map_err(|_| FontError::Bounds)?, 4)?,
        );
        Ok(tag)
    }
    fn words(&self, at: u64) -> std::result::Result<Vec<u16>, FontError> {
        let count = self.word(at)?;
        let mut values = Vec::new();
        values.try_reserve_exact(usize::from(count))?;
        for index in 0..count {
            values.push(self.word(at + 2 + u64::from(index) * 2)?);
        }
        Ok(values)
    }
    fn language(&self, at: u64) -> std::result::Result<Vec<u16>, FontError> {
        self.word(at)?;
        self.word(at + 2)?;
        self.words(at + 4)
    }
    pub(super) fn scripts(&self, root: u64) -> std::result::Result<Scripts, FontError> {
        let mut scripts = Vec::new();
        if let Some(list) = self.pointer(root, 4)? {
            let count = self.word(list)?;
            scripts.try_reserve_exact(usize::from(count))?;
            for index in 0..count {
                let entry = 2 + u64::from(index) * 6;
                let tag = self.tag(list + entry)?;
                let mut default = None;
                if let Some(script) = self.pointer(list, entry + 4)? {
                    if let Some(language) = self.pointer(script, 0)? {
                        default = Some(self.language(language)?);
                    }
                    let count = self.word(script + 2)?;
                    for index in 0..count {
                        let entry = 4 + u64::from(index) * 6;
                        self.tag(script + entry)?;
                        if let Some(language) = self.pointer(script, entry + 4)? {
                            self.language(language)?;
                        }
                    }
                }
                scripts.push((tag, default));
            }
        }
        Ok(scripts)
    }
    pub(super) fn features(&self, root: u64) -> std::result::Result<Features, FontError> {
        let mut features = Vec::new();
        if let Some(list) = self.pointer(root, 6)? {
            let count = self.word(list)?;
            features.try_reserve_exact(usize::from(count))?;
            for index in 0..count {
                let entry = 2 + u64::from(index) * 6;
                let tag = self.tag(list + entry)?;
                let feature = self
                    .pointer(list, entry + 4)?
                    .map(|at| {
                        self.word(at)?;
                        self.words(at + 2)
                    })
                    .transpose()?;
                features.push((tag, feature));
            }
        }
        Ok(features)
    }
    pub(super) fn lookups(&mut self, root: u64) -> Result<Vec<Option<u16>>, E> {
        let mut lookups = Vec::new();
        if let Some(list) = self.pointer(root, 8)? {
            let count = self.word(list)?;
            lookups
                .try_reserve_exact(usize::from(count))
                .map_err(FontError::from)?;
            for index in 0..count {
                let mut kind = None;
                if let Some(lookup) = self.pointer(list, 2 + u64::from(index) * 2)? {
                    let value = self.word(lookup)?;
                    if !(1..=9).contains(&value) {
                        return Err(FontError::GposLookupType(value).into());
                    }
                    kind = Some(value);
                    let flags = self.word(lookup + 2)?;
                    let count = self.word(lookup + 4)?;
                    for index in 0..count {
                        // A subtable warning can change a later pointer. The
                        // already parsed kind/flags/count remain snapshots.
                        if let Some(subtable) = self.pointer(lookup, 6 + u64::from(index) * 2)? {
                            self.subtable(subtable, value)?;
                        }
                    }
                    if flags & 16 != 0 {
                        self.word(lookup + 6 + u64::from(count) * 2)?;
                    }
                }
                lookups.push(kind);
            }
        }
        Ok(lookups)
    }
}

pub(super) fn initialize(
    scripts: Scripts,
    features: Features,
    lookups: Vec<Option<u16>>,
) -> std::result::Result<(), FontError> {
    let script = if scripts.iter().any(|(tag, _)| tag == b"DFLT") {
        *b"DFLT"
    } else if scripts.iter().any(|(tag, _)| tag == b"latn") {
        *b"latn"
    } else {
        *b"DFLT"
    };
    // Upstream searches the ordered script records with its binary search,
    // including malformed unsorted inputs. Do not substitute a linear search.
    let (mut low, mut high) = (0, scripts.len());
    let mut selected = None;
    while low < high {
        let middle = (low + high - 1) / 2;
        match scripts[middle].0.cmp(&script) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle,
            std::cmp::Ordering::Equal => {
                selected = scripts[middle].1.as_ref();
                break;
            }
        }
    }
    if let Some(indices) = selected {
        for index in indices {
            let (tag, feature) = features.get(usize::from(*index)).ok_or(FontError::Type(
                "Cannot read properties of undefined (reading 'tag')",
            ))?;
            if tag != b"kern" {
                continue;
            }
            if let Some(indices) = feature {
                for index in indices {
                    lookups
                        .get(usize::from(*index))
                        .copied()
                        .flatten()
                        .ok_or(FontError::Type(
                            "Cannot read properties of undefined (reading 'lookupType')",
                        ))?;
                }
            }
            break;
        }
    }
    Ok(())
}
