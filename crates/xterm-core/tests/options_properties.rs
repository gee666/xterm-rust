use std::{
    cell::Cell,
    rc::{Rc, Weak},
};
use xterm_core::options::{OptionObject, OptionValue as V, OptionsService, WindowsPty};

#[test]
fn scrollbar_mutation_is_shared_by_existing_and_new_services() {
    let first = OptionsService::default();
    let second = OptionsService::default();
    let calls = Rc::new(Cell::new(0));
    let output = calls.clone();
    first
        .on_any_option_change()
        .subscribe(move |_| output.set(output.get() + 1));
    let scrollbar = first.get("scrollbar").unwrap();
    let original = scrollbar.get_property("showScrollbar").unwrap();
    scrollbar
        .set_property("showScrollbar", V::Bool(false))
        .unwrap();
    assert_eq!(
        second
            .get("scrollbar")
            .unwrap()
            .get_property("showScrollbar")
            .unwrap(),
        V::Bool(false)
    );
    assert_eq!(
        OptionsService::default()
            .get("scrollbar")
            .unwrap()
            .get_property("showScrollbar")
            .unwrap(),
        V::Bool(false)
    );
    assert_eq!(calls.get(), 0);
    first
        .set(
            "scrollbar",
            V::property_map([("showScrollbar", V::Bool(true))]),
        )
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(
        second
            .get("scrollbar")
            .unwrap()
            .get_property("showScrollbar")
            .unwrap(),
        V::Bool(false)
    );
    scrollbar.set_property("showScrollbar", original).unwrap();
}

#[test]
fn every_default_data_bag_supports_live_shared_properties() {
    let first = OptionsService::default();
    let second = OptionsService::default();
    for (group, key, value) in [
        ("theme", "foreground", V::from("#aabbcc")),
        ("windowOptions", "getWinSizePixels", V::Bool(false)),
        ("quirks", "allowSetCursorBlink", V::Bool(true)),
        ("vtExtensions", "kittyKeyboard", V::Bool(true)),
        ("windowsPty", "backend", V::from("conpty")),
    ] {
        let bag = first.get(group).unwrap();
        assert_eq!(bag.get_property(key).unwrap(), V::Undefined);
        bag.set_property(key, value.clone()).unwrap();
        assert_eq!(second.get(group).unwrap().get_property(key).unwrap(), value);
        assert_eq!(
            OptionsService::default()
                .get(group)
                .unwrap()
                .get_property(key)
                .unwrap(),
            value
        );
        assert!(bag.delete_property(key).unwrap());
        assert_eq!(
            second.get(group).unwrap().get_property(key).unwrap(),
            V::Undefined
        );
    }
}

#[test]
fn windows_pty_builder_is_a_mutable_data_bag() {
    let bag = V::from(WindowsPty {
        backend: Some("conpty".into()),
        build_number: Some(0.0),
    });
    bag.set_property("buildNumber", V::from("0")).unwrap();
    assert_eq!(bag.get_property("buildNumber").unwrap(), V::from("0"));
    bag.delete_property("backend").unwrap();
    assert_eq!(bag.get_property("backend").unwrap(), V::Undefined);
}

#[test]
fn property_order_replacement_and_child_aliases_match_data_objects() {
    let child = V::empty_object();
    let parent = V::property_map([("b", V::Bool(false)), ("a", child.clone())]);
    parent.set_property("2", V::Number(2.0)).unwrap();
    parent.set_property("1", V::Number(1.0)).unwrap();
    parent.set_property("b", V::Bool(true)).unwrap();
    let V::Object(object) = &parent else { panic!() };
    assert_eq!(object.property_keys().unwrap(), ["1", "2", "b", "a"]);
    parent
        .get_property("a")
        .unwrap()
        .set_property("flag", V::Bool(true))
        .unwrap();
    assert_eq!(child.get_property("flag").unwrap(), V::Bool(true));
    assert!(parent.delete_property("missing").unwrap());
}

#[test]
fn replaced_property_payload_can_reenter_the_same_bag() {
    struct Probe(Weak<OptionObject>);
    impl Drop for Probe {
        fn drop(&mut self) {
            let object = self.0.upgrade().unwrap();
            assert_eq!(object.get_property("value").unwrap(), V::Null);
            object.set_property("sideEffect", V::Bool(true)).unwrap();
        }
    }
    let bag = V::empty_object();
    let V::Object(object) = &bag else { panic!() };
    bag.set_property("value", V::object(Probe(Rc::downgrade(object))))
        .unwrap();
    bag.set_property("value", V::Null).unwrap();
    assert_eq!(bag.get_property("sideEffect").unwrap(), V::Bool(true));
}

#[test]
fn native_data_cycles_are_rejected_without_leaking_or_partial_writes() {
    let parent = V::empty_object();
    let child = V::empty_object();
    parent.set_property("child", child.clone()).unwrap();
    assert!(child.set_property("parent", parent).is_err());
    assert_eq!(child.get_property("parent").unwrap(), V::Undefined);
}
