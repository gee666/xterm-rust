// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_addons::search::{
    SearchColor, SearchDecorationOptions, SearchDecorationOptionsHandle, SearchDecorationSource,
};
use xterm_core::event::EmitError;

struct MutableColors {
    value: RefCell<SearchDecorationOptions>,
    borders: Cell<usize>,
    failure: RefCell<Option<EmitError>>,
}
impl SearchDecorationSource for MutableColors {
    fn color(&self, kind: SearchColor) -> Result<Option<String>, EmitError> {
        if matches!(kind, SearchColor::MatchBorder | SearchColor::ActiveBorder) {
            self.borders.set(self.borders.get() + 1);
            if let Some(error) = self.failure.borrow().clone() {
                return Err(error);
            }
        }
        self.value.borrow().color(kind)
    }
}
fn mutable_colors() -> Rc<MutableColors> {
    Rc::new(MutableColors {
        value: RefCell::new(SearchDecorationOptions {
            match_border: Some("#ff0000".into()),
            active_match_border: Some("#ff0000".into()),
            match_overview_ruler: "#ff0000".into(),
            active_match_overview_ruler: "#00ff00".into(),
            ..SearchDecorationOptions::default()
        }),
        borders: Cell::new(0),
        failure: RefCell::new(None),
    })
}

#[test]
fn borders_are_read_live_at_each_render_even_after_first_outline_assignment() {
    let h = Harness::new(20, 3, None);
    h.write("hello");
    let colors = mutable_colors();
    let source: SearchDecorationOptionsHandle = colors.clone();
    h.addon
        .find_next(
            &term("hello"),
            Some(Rc::new(RefCell::new(SearchOptions {
                decorations: Some(source),
                ..SearchOptions::default()
            }))),
        )
        .unwrap();
    assert_eq!(colors.borders.get(), 0);
    colors.value.borrow_mut().match_border = Some("#0000ff".into());
    colors.value.borrow_mut().active_match_border = Some("#0000ff".into());
    let elements: Vec<_> = h
        .decorations()
        .into_iter()
        .map(|decoration| {
            let element = Rc::new(Element::default());
            let opaque: Rc<dyn std::any::Any> = element.clone();
            decoration.fire_render(opaque).unwrap();
            assert_eq!(
                element.outline.borrow().as_deref(),
                Some("1px solid #0000ff")
            );
            (decoration, element)
        })
        .collect();
    assert_eq!(colors.borders.get(), 2);
    colors.value.borrow_mut().match_border = Some("#00ff00".into());
    colors.value.borrow_mut().active_match_border = Some("#00ff00".into());
    for (decoration, element) in elements {
        let opaque: Rc<dyn std::any::Any> = element.clone();
        decoration.fire_render(opaque).unwrap();
        assert_eq!(
            element.outline.borrow().as_deref(),
            Some("1px solid #0000ff")
        );
    }
    assert_eq!(colors.borders.get(), 4);
}

#[test]
fn a_throwing_live_border_getter_retains_error_identity_and_existing_style() {
    let h = Harness::new(20, 3, None);
    h.write("hello");
    let colors = mutable_colors();
    let source: SearchDecorationOptionsHandle = colors.clone();
    h.addon
        .find_next(
            &term("hello"),
            Some(Rc::new(RefCell::new(SearchOptions {
                decorations: Some(source),
                ..SearchOptions::default()
            }))),
        )
        .unwrap();
    let decoration = h.decorations()[0].clone();
    let element = Rc::new(Element::default());
    let opaque: Rc<dyn std::any::Any> = element.clone();
    decoration.fire_render(opaque.clone()).unwrap();
    let sentinel = EmitError::callback(std::io::Error::other("border getter sentinel"));
    *colors.failure.borrow_mut() = Some(sentinel.clone());
    assert_eq!(decoration.fire_render(opaque), Err(sentinel));
    assert_eq!(
        element.outline.borrow().as_deref(),
        Some("1px solid #ff0000")
    );
}
