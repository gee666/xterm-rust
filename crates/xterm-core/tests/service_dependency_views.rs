// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::rc::Rc;
use xterm_core::{
    instantiation::{InstantiationService, ServiceValue},
    service_registry::{ConstructorMetadata, Dependency, ServiceRegistry},
};

#[test]
fn unannotated_views_are_fresh_and_never_attach_to_metadata() {
    let mut registry = ServiceRegistry::default();
    let id = registry.create_decorator::<String>("value").unwrap();
    let base = ConstructorMetadata::new("Base", None);
    let child = ConstructorMetadata::new("Child", Some(base.clone()));
    let first = child.dependencies_view();
    let second = child.dependencies_view();
    assert!(!Rc::ptr_eq(&first, &second));
    first.borrow_mut().push(Dependency {
        id: id.key(),
        index: 0,
    });
    assert!(base.dependencies().is_empty());
    assert!(child.dependencies().is_empty());
    let service = InstantiationService::new(&mut registry).unwrap();
    service
        .create_instance(&child, vec![], |args| assert!(args.is_empty()))
        .unwrap();
    base.decorate(id.key(), 0, 3).unwrap();
    assert!(!Rc::ptr_eq(&first, &child.dependencies_view()));
    assert_eq!(first.borrow().len(), 1);
}

#[test]
fn inherited_views_alias_until_the_first_valid_own_decorator() {
    let mut registry = ServiceRegistry::default();
    let a = registry.create_decorator::<String>("a").unwrap();
    let b = registry.create_decorator::<String>("b").unwrap();
    let base = ConstructorMetadata::new("Base", None);
    base.decorate(a.key(), 2, 3).unwrap();
    let child = ConstructorMetadata::new("Child", Some(base.clone()));
    let grandchild = ConstructorMetadata::new("Grandchild", Some(child.clone()));
    let inherited = grandchild.dependencies_view();
    assert!(Rc::ptr_eq(&inherited, &base.dependencies_view()));
    assert!(Rc::ptr_eq(&inherited, &child.dependencies_view()));
    inherited.borrow_mut()[0] = Dependency {
        id: b.key(),
        index: 0,
    };
    assert_eq!(base.dependencies(), child.dependencies());
    assert!(child.decorate(a.key(), 1, 2).is_err());
    assert!(Rc::ptr_eq(&inherited, &child.dependencies_view()));
    child.decorate(a.key(), 1, 3).unwrap();
    let own = child.dependencies_view();
    assert!(!Rc::ptr_eq(&inherited, &own));
    assert!(Rc::ptr_eq(&own, &grandchild.dependencies_view()));
    inherited.borrow_mut().clear();
    assert!(base.dependencies().is_empty());
    assert_eq!(
        grandchild.dependencies(),
        [Dependency {
            id: a.key(),
            index: 1
        }]
    );
    own.borrow_mut().clear();
    child.decorate(b.key(), 0, 3).unwrap();
    assert!(Rc::ptr_eq(&own, &child.dependencies_view()));
    assert_eq!(
        own.borrow().as_slice(),
        &[Dependency {
            id: b.key(),
            index: 0
        }]
    );
}

#[test]
fn live_edits_affect_injection_errors_sorting_and_constructor_calls() {
    let mut registry = ServiceRegistry::default();
    let a = registry.create_decorator::<String>("a").unwrap();
    let b = registry.create_decorator::<String>("b").unwrap();
    let service = InstantiationService::new(&mut registry).unwrap();
    service.set_service(&a, Rc::new("A".to_owned()));
    let base = ConstructorMetadata::new("Base", None);
    base.decorate(a.key(), 3, 3).unwrap();
    let child = ConstructorMetadata::new("Child", Some(base.clone()));
    let view = child.dependencies_view();
    view.borrow_mut().extend([
        Dependency {
            id: b.key(),
            index: 0,
        },
        Dependency {
            id: a.key(),
            index: 0,
        },
    ]);
    let error = service
        .create_instance(&child, vec![], |_| panic!("validation must fail"))
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "[createInstance] Child depends on UNKNOWN service b."
    );
    assert_eq!(
        view.borrow().iter().map(|d| d.index).collect::<Vec<_>>(),
        [0, 0, 3]
    );
    service.set_service(&b, Rc::new("B".to_owned()));
    let values = service
        .create_instance(&child, vec![], |args| {
            // A live view can be mutated from the constructor with no metadata/list borrow held.
            view.borrow_mut().clear();
            args.into_iter()
                .map(|value| (*value.downcast::<String>().unwrap()).clone())
                .collect::<Vec<_>>()
        })
        .unwrap();
    assert_eq!(values, ["B", "A", "A"]);
    assert!(base.dependencies().is_empty());
    service
        .create_instance(
            &child,
            vec![ServiceValue::new("static".to_owned())],
            |args| assert_eq!(args.len(), 1),
        )
        .unwrap();
    assert!(Rc::ptr_eq(&view, &base.dependencies_view()));
}
