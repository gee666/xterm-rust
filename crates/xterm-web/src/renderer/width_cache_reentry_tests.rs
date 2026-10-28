use super::*;
use std::cell::{Cell, RefCell};
type Configure = Rc<dyn Fn(&Font, FontVariant) -> Result<(), &'static str>>;
type Measure = Rc<dyn Fn()>;
#[derive(Default)]
struct Driver {
    configured: RefCell<Vec<(String, FontVariant)>>,
    on_configure: RefCell<Option<Configure>>,
    on_measure: RefCell<Option<Measure>>,
    width: Cell<f64>,
    measured: Cell<usize>,
}
struct Measurer(Rc<Driver>);
impl WidthMeasurer for Measurer {
    type Font = Font;
    type Error = &'static str;
    fn configure_variant(&self, font: &Font, variant: FontVariant) -> Result<(), Self::Error> {
        self.0
            .configured
            .borrow_mut()
            .push((font.family.clone(), variant));
        let callback = self.0.on_configure.borrow().clone();
        if let Some(callback) = callback {
            callback(font, variant)?;
        }
        Ok(())
    }
    fn measure(&self, _: &[u16], _: FontVariant) -> Result<f64, Self::Error> {
        self.0.measured.set(self.0.measured.get() + 1);
        let width = self.0.width.get();
        let callback = self.0.on_measure.borrow().clone();
        if let Some(callback) = callback {
            callback();
        }
        Ok(width)
    }
}
fn font(name: &str) -> Font {
    Font {
        family: name.into(),
        size: 15.0,
        weight: "normal".into(),
        weight_bold: "bold".into(),
    }
}
fn make() -> (WidthCache<Measurer>, Rc<Driver>) {
    let driver = Rc::new(Driver::default());
    driver.width.set(3.0);
    let cache = WidthCache::new(Measurer(Rc::clone(&driver)));
    cache.set_font(font("old")).unwrap();
    driver.configured.borrow_mut().clear();
    (cache, driver)
}
#[test]
fn failed_configuration_commits_key_but_does_not_clear_or_retry() {
    let (cache, driver) = make();
    assert_eq!(cache.get(&[97], false, false).unwrap(), 3.0);
    *driver.on_configure.borrow_mut() = Some(Rc::new(|_, variant| {
        if variant == FontVariant::Bold {
            Err("font setter")
        } else {
            Ok(())
        }
    }));
    assert_eq!(
        cache.set_font(font("new")),
        Err(WidthError::Measure("font setter"))
    );
    assert_eq!(driver.configured.borrow().len(), 2);
    assert_eq!(cache.state.borrow().font.family, "new");
    cache.set_font(font("new")).unwrap();
    assert_eq!(driver.configured.borrow().len(), 2);
    assert_eq!(cache.get(&[97], false, false).unwrap(), 3.0);
}
#[test]
fn disposal_in_variant_callback_prevents_next_variant_lookup() {
    let (cache, driver) = make();
    cache.get(&[97], false, false).unwrap();
    let weak = cache.downgrade();
    *driver.on_configure.borrow_mut() = Some(Rc::new(move |_, _| {
        weak.upgrade().unwrap().dispose();
        Ok(())
    }));
    assert_eq!(cache.set_font(font("new")), Err(WidthError::Disposed));
    assert_eq!(
        *driver.configured.borrow(),
        [("new".to_owned(), FontVariant::Regular)]
    );
    assert_eq!(cache.get(&[97], false, false).unwrap(), 3.0);
}
#[test]
fn reentrant_set_font_keeps_inner_key_and_outer_captured_arguments() {
    let (cache, driver) = make();
    let weak = cache.downgrade();
    let entered = Cell::new(false);
    *driver.on_configure.borrow_mut() = Some(Rc::new(move |font, variant| {
        if font.family == "outer" && variant == FontVariant::Regular && !entered.replace(true) {
            weak.upgrade()
                .unwrap()
                .set_font(super::reentry_tests::font("inner"))
                .unwrap();
        }
        Ok(())
    }));
    cache.set_font(font("outer")).unwrap();
    assert_eq!(cache.state.borrow().font.family, "inner");
    use FontVariant::*;
    assert_eq!(
        *driver.configured.borrow(),
        [
            ("outer".into(), Regular),
            ("inner".into(), Regular),
            ("inner".into(), Bold),
            ("inner".into(), Italic),
            ("inner".into(), BoldItalic),
            ("outer".into(), Bold),
            ("outer".into(), Italic),
            ("outer".into(), BoldItalic)
        ]
    );
}
#[test]
fn clear_during_measure_stores_in_the_new_current_map() {
    let (cache, driver) = make();
    let weak = cache.downgrade();
    *driver.on_measure.borrow_mut() = Some(Rc::new(move || weak.upgrade().unwrap().clear()));
    assert_eq!(cache.get(&[256], false, false).unwrap(), 3.0);
    *driver.on_measure.borrow_mut() = None;
    driver.width.set(8.0);
    assert_eq!(cache.get(&[256], false, false).unwrap(), 3.0);
    assert_eq!(driver.measured.get(), 1);
}
#[test]
fn dispose_during_measure_retains_positive_flat_but_rejects_positive_holey_store() {
    for text in [vec![97], vec![256]] {
        let (cache, driver) = make();
        let weak = cache.downgrade();
        *driver.on_measure.borrow_mut() = Some(Rc::new(move || weak.upgrade().unwrap().dispose()));
        if text == [97] {
            assert_eq!(cache.get(&text, false, false).unwrap(), 3.0);
            assert_eq!(cache.get(&text, false, false).unwrap(), 3.0);
        } else {
            assert_eq!(cache.get(&text, false, false), Err(WidthError::Disposed));
        }
    }
    for width in [0.0, -1.0, f64::NAN] {
        let (cache, driver) = make();
        driver.width.set(width);
        let weak = cache.downgrade();
        *driver.on_measure.borrow_mut() = Some(Rc::new(move || weak.upgrade().unwrap().dispose()));
        let value = cache.get(&[256], false, false).unwrap();
        assert!(value == width || (value.is_nan() && width.is_nan()));
    }
}
#[test]
fn weak_callback_handles_do_not_keep_cache_alive() {
    let (cache, driver) = make();
    let weak = cache.downgrade();
    let callback = weak.clone();
    *driver.on_measure.borrow_mut() = Some(Rc::new(move || {
        assert!(callback.upgrade().is_none());
    }));
    drop(cache);
    assert!(weak.upgrade().is_none());
    driver.on_measure.borrow().as_ref().unwrap()();
}
