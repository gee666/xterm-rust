use super::*;
use std::{
    cell::{Cell, RefCell},
    convert::Infallible,
    rc::Rc,
};

struct Mock {
    calls: Rc<Cell<usize>>,
    configured: Rc<RefCell<Vec<Font>>>,
    width: f64,
}
impl WidthMeasurer for Mock {
    type Error = Infallible;
    fn configure(&mut self, font: &Font) -> Result<(), Infallible> {
        self.configured.borrow_mut().push(font.clone());
        Ok(())
    }
    fn measure(&mut self, _: &[u16], _: FontVariant) -> Result<f64, Infallible> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.width)
    }
}
fn font() -> Font {
    Font {
        family: "monospace".into(),
        size: 15.0,
        weight: "normal".into(),
        weight_bold: "bold".into(),
    }
}
fn cache(width: f64) -> WidthCache<Mock> {
    let mut cache = WidthCache::new(Mock {
        calls: Rc::default(),
        configured: Rc::default(),
        width,
    });
    cache.set_font(font()).unwrap();
    cache
}
fn populated() -> WidthCache<Mock> {
    let mut cache = cache(5.0);
    cache.flat.fill(1.23);
    cache.holey.as_mut().unwrap().insert(vec![97], 2.34);
    cache
}
fn cleared(cache: &WidthCache<Mock>) {
    assert_eq!(cache.flat[0], FLAT_UNSET);
    assert_eq!(cache.holey.as_ref().unwrap().get(&vec![97]), None);
    assert!(cache.holey.as_ref().unwrap().is_empty());
}

#[test]
fn upstream_cache_values_and_clear() {
    let mut cache = populated();
    assert_eq!(cache.flat[0], 1.23_f32);
    assert_eq!(cache.holey.as_ref().unwrap().get(&vec![97]), Some(&2.34));
    assert_eq!(cache.holey.as_ref().unwrap().len(), 1);
    cache.clear();
    cleared(&cache);
}
#[test]
fn upstream_every_font_field_invalidates_and_unchanged_font_retains_entries() {
    for changed in [
        Font {
            family: "Arial".into(),
            ..font()
        },
        Font {
            size: 14.0,
            ..font()
        },
        Font {
            weight: "100".into(),
            ..font()
        },
        Font {
            weight_bold: "900".into(),
            ..font()
        },
    ] {
        let mut cache = populated();
        cache.set_font(changed).unwrap();
        cleared(&cache);
    }
    let mut cache = populated();
    cache.set_font(font()).unwrap();
    assert_eq!(cache.flat[0], 1.23_f32);
    assert_eq!(cache.holey.as_ref().unwrap().get(&vec![97]), Some(&2.34));
    assert_eq!(cache.holey.as_ref().unwrap().len(), 1);
}
#[test]
fn upstream_flat_and_holey_boundary() {
    let mut cache = cache(5.0);
    for index in 0..FLAT_SIZE + 10 {
        let text = [index as u16];
        assert_eq!(cache.get(&text, false, false).unwrap(), 5.0);
        if index < FLAT_SIZE {
            assert_eq!(cache.flat[index], 5.0);
            assert_eq!(cache.holey.as_ref().unwrap().get(text.as_slice()), None);
        } else {
            assert_eq!(
                cache.holey.as_ref().unwrap().get(text.as_slice()),
                Some(&5.0)
            );
        }
    }
}
#[test]
fn upstream_font_variants_and_arbitrary_strings() {
    let mut cache = cache(5.0);
    for (text, bold, italic, key) in [
        ("b", true, false, "bB"),
        ("i", false, true, "iI"),
        ("x", true, true, "xBI"),
        ("foo", false, false, "foo"),
        ("bar&baz", true, true, "bar&bazBI"),
    ] {
        let text: Vec<_> = text.encode_utf16().collect();
        let key: Vec<_> = key.encode_utf16().collect();
        assert_eq!(cache.get(&text, bold, italic).unwrap(), 5.0);
        assert_eq!(cache.holey.as_ref().unwrap().get(&key), Some(&5.0));
    }
}
#[test]
fn flat_values_round_only_on_subsequent_reads() {
    let mut cache = cache(1.23456789);
    assert_eq!(cache.get(&[97], false, false).unwrap(), 1.23456789);
    assert_eq!(
        cache.get(&[97], false, false).unwrap(),
        f64::from(1.23456789_f32)
    );
    assert_eq!(cache.measurer.as_ref().unwrap().calls.get(), 1);
}
#[test]
fn nonpositive_nan_values_are_remeasured_and_suffix_collisions_are_retained() {
    for width in [0.0, -1.0, f64::NAN] {
        let mut cache = cache(width);
        for text in [vec![97], vec![256], vec![]] {
            let _ = cache.get(&text, false, false).unwrap();
            let _ = cache.get(&text, false, false).unwrap();
        }
        assert_eq!(cache.measurer.as_ref().unwrap().calls.get(), 6);
    }
    let mut cache = cache(3.0);
    cache.get(&[97], true, false).unwrap();
    cache.measurer.as_mut().unwrap().width = 9.0;
    assert_eq!(cache.get(&[97, 66], false, false).unwrap(), 3.0);
}
#[test]
fn disposed_cache_retains_cached_flat_values_but_cannot_measure() {
    let mut cache = cache(3.0);
    cache.get(&[97], false, false).unwrap();
    cache.dispose();
    assert_eq!(cache.get(&[97], false, false).unwrap(), 3.0);
    assert_eq!(cache.get(&[98], false, false), Err(WidthError::Disposed));
    assert_eq!(cache.get(&[256], false, false), Err(WidthError::Disposed));
    cache.clear();
    assert_eq!(cache.get(&[97], false, false), Err(WidthError::Disposed));
}
