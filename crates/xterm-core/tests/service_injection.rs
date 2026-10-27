use std::cell::Cell;
use std::rc::Rc;
use xterm_core::instantiation::{
    EmptyValue, InstantiationService, ServiceCollection, ServiceValue,
};
use xterm_core::service_registry::{ConstructorMetadata, RegistryError, ServiceRegistry};

#[test]
fn registry_identity_names_and_type_boundaries() {
    let mut registry = ServiceRegistry::default();
    for name in ["", "name", "🦀"] {
        let first = registry.create_decorator::<String>(name).unwrap();
        let second = registry.create_decorator::<String>(name).unwrap();
        assert_eq!(first.key(), second.key());
        assert_eq!(first.key().name(), name);
        assert!(matches!(
            registry.create_decorator::<u32>(name),
            Err(RegistryError::TypeMismatch(_))
        ));
    }
    let mut other = ServiceRegistry::default();
    assert_ne!(
        registry.create_decorator::<String>("").unwrap().key(),
        other.create_decorator::<String>("").unwrap().key()
    );
}

#[test]
fn inherited_metadata_replacement_arity_and_stable_sort() {
    let mut registry = ServiceRegistry::default();
    let a = registry.create_decorator::<String>("a").unwrap();
    let b = registry.create_decorator::<String>("b").unwrap();
    let base = ConstructorMetadata::new("Base", None);
    assert!(base.dependencies().is_empty());
    base.decorate(a.key(), 2, 3).unwrap();
    base.decorate(b.key(), 0, 3).unwrap();
    let child = ConstructorMetadata::new("Child", Some(base.clone()));
    assert_eq!(child.dependencies(), base.dependencies());
    for arity in [0, 1, 2, 4] {
        assert_eq!(
            child.decorate(a.key(), 1, arity).unwrap_err().to_string(),
            "@IServiceName-decorator can only be used to decorate a parameter"
        );
        assert_eq!(child.dependencies(), base.dependencies());
    }
    let service = InstantiationService::new(&mut registry).unwrap();
    service.set_service(&a, Rc::new("a".to_owned()));
    service.set_service(&b, Rc::new("b".to_owned()));
    service.create_instance(&child, vec![], |_| ()).unwrap();
    assert_eq!(base.dependencies()[0].id, b.key());
    child.decorate(a.key(), 0, 3).unwrap();
    child.decorate(b.key(), 0, 3).unwrap();
    child.decorate(a.key(), 7, 3).unwrap();
    assert_eq!(child.dependencies().len(), 3);
    let values = service
        .create_instance(&child, vec![], |args| {
            args.iter()
                .map(|v| (*v.downcast::<String>().unwrap()).clone())
                .collect::<Vec<_>>()
        })
        .unwrap();
    assert_eq!(values, ["a", "b", "a"]);
    assert_eq!(base.dependencies().len(), 2);
}

#[test]
fn collection_replacement_identity_and_initial_entries() {
    let mut registry = ServiceRegistry::default();
    let id = registry.create_decorator::<String>("id").unwrap();
    let collection = ServiceCollection::default();
    assert!(!collection.has(&id.key()));
    assert!(collection.get(&id).is_none());
    let first = Rc::new("first".to_owned());
    assert!(collection.set(&id, first.clone()).is_none());
    assert!(Rc::ptr_eq(&collection.get(&id).unwrap(), &first));
    let second = Rc::new("second".to_owned());
    assert!(Rc::ptr_eq(
        &collection.set(&id, second.clone()).unwrap(),
        &first
    ));
    assert!(Rc::ptr_eq(&collection.get(&id).unwrap(), &second));
    let collection = ServiceCollection::new([
        (id.key(), ServiceValue::shared(first)),
        (id.key(), ServiceValue::shared(second.clone())),
    ])
    .unwrap();
    assert!(Rc::ptr_eq(&collection.get(&id).unwrap(), &second));
    assert!(collection
        .set_value(id.key(), ServiceValue::new(4_u32))
        .is_err());
    assert!(Rc::ptr_eq(&collection.get(&id).unwrap(), &second));
}

#[test]
fn collection_iteration_is_live_reentrant_and_insertion_ordered() {
    let mut registry = ServiceRegistry::default();
    let ids: Vec<_> = ["a", "b", "c"]
        .map(|s| registry.create_decorator::<String>(s).unwrap())
        .into();
    let collection = ServiceCollection::default();
    collection.set(&ids[0], Rc::new("a0".to_owned()));
    collection.set(&ids[1], Rc::new("b0".to_owned()));
    let mut trace = Vec::new();
    collection.for_each(|key, value| {
        trace.push((
            key.name().to_owned(),
            (*value.downcast::<String>().unwrap()).clone(),
        ));
        if key == ids[0].key() {
            collection.set(&ids[0], Rc::new("a1".to_owned()));
            collection.set(&ids[1], Rc::new("b1".to_owned()));
            collection.set(&ids[2], Rc::new("c1".to_owned()));
            let count = Cell::new(0);
            collection.for_each(|_, _| count.set(count.get() + 1));
            assert_eq!(count.get(), 3);
        }
    });
    assert_eq!(
        trace,
        [
            ("a".into(), "a0".into()),
            ("b".into(), "b1".into()),
            ("c".into(), "c1".into())
        ]
    );
}

#[test]
fn self_registration_replacement_and_cycle_free_drop() {
    let mut registry = ServiceRegistry::default();
    let first = InstantiationService::new(&mut registry).unwrap();
    let id = registry
        .create_decorator::<InstantiationService>("InstantiationService")
        .unwrap();
    assert!(Rc::ptr_eq(&first, &first.get_service(&id).unwrap()));
    first.set_service(&id, first.clone());
    assert!(Rc::ptr_eq(&first, &first.get_service(&id).unwrap()));
    let second = InstantiationService::new(&mut registry).unwrap();
    first.set_service(&id, second.clone());
    assert!(Rc::ptr_eq(&second, &first.get_service(&id).unwrap()));
    let weak_first = Rc::downgrade(&first);
    let weak_second = Rc::downgrade(&second);
    drop(first);
    drop(second);
    assert!(weak_first.upgrade().is_none());
    assert!(weak_second.upgrade().is_none());
}

#[test]
fn collection_callback_failure_keeps_prior_mutations_and_stops() {
    let mut registry = ServiceRegistry::default();
    let a = registry.create_decorator::<String>("a").unwrap();
    let b = registry.create_decorator::<String>("b").unwrap();
    let collection = ServiceCollection::default();
    collection.set(&a, Rc::new("a".into()));
    collection.set(&b, Rc::new("b".into()));
    let mut count = 0;
    let result = collection.try_for_each(|_, _| {
        count += 1;
        collection.set(&b, Rc::new("changed".into()));
        Err::<(), _>("failure")
    });
    assert_eq!(result, Err("failure"));
    assert_eq!(count, 1);
    assert_eq!(*collection.get(&b).unwrap(), "changed");
}

fn resolves<T: 'static>(value: T) -> bool {
    let mut registry = ServiceRegistry::default();
    let service = InstantiationService::new(&mut registry).unwrap();
    let id = registry.create_decorator::<T>("value").unwrap();
    service.set_service(&id, Rc::new(value));
    let ctor = ConstructorMetadata::new("Ctor", None);
    ctor.decorate(id.key(), 0, 3).unwrap();
    service.create_instance(&ctor, vec![], |_| ()).is_ok()
}

#[test]
fn primitive_truthiness_and_objects() {
    assert!(!resolves(false));
    assert!(resolves(true));
    assert!(!resolves(0.0_f64));
    assert!(!resolves(-0.0_f64));
    assert!(!resolves(f64::NAN));
    assert!(resolves(f64::INFINITY));
    assert!(!resolves(f32::NAN));
    assert!(!resolves(0_u32));
    assert!(!resolves(0_i64));
    assert!(resolves(-1_i32));
    assert!(!resolves(String::new()));
    assert!(!resolves(""));
    assert!(resolves("x".to_owned()));
    assert!(!resolves(EmptyValue::Null));
    assert!(!resolves(EmptyValue::Undefined));
    assert!(resolves(Vec::<u8>::new()));
}

#[test]
fn missing_service_precedes_static_count_and_never_calls_constructor() {
    let mut registry = ServiceRegistry::default();
    let service = InstantiationService::new(&mut registry).unwrap();
    let id = registry.create_decorator::<String>("missing").unwrap();
    let ctor = ConstructorMetadata::new("Ctor", None);
    ctor.decorate(id.key(), 2, 3).unwrap();
    let called = Cell::new(false);
    let error = service
        .create_instance(&ctor, vec![], |_| called.set(true))
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "[createInstance] Ctor depends on UNKNOWN service missing."
    );
    assert!(!called.get());
}

#[test]
fn argument_errors_order_gaps_no_dependencies_and_callback_failures() {
    let mut registry = ServiceRegistry::default();
    let service = InstantiationService::new(&mut registry).unwrap();
    let id = registry.create_decorator::<String>("text").unwrap();
    service.set_service(&id, Rc::new("service".to_owned()));
    let ctor = ConstructorMetadata::new("Ctor", None);
    ctor.decorate(id.key(), 1, 3).unwrap();
    ctor.decorate(id.key(), 9, 3).unwrap();
    for count in [0, 2] {
        let args = (0..count).map(ServiceValue::new).collect();
        let error = service.create_instance(&ctor, args, |_| ()).unwrap_err();
        assert_eq!(error.to_string(), format!("[createInstance] First service dependency of Ctor at position 2 conflicts with {count} static arguments"));
    }
    let args = service
        .create_instance(
            &ctor,
            vec![ServiceValue::new("static".to_owned())],
            |args| args,
        )
        .unwrap();
    assert_eq!(args.len(), 3);
    assert_eq!(*args[0].downcast::<String>().unwrap(), "static");
    assert_eq!(*args[1].downcast::<String>().unwrap(), "service");
    assert!(Rc::ptr_eq(
        &args[1].downcast::<String>().unwrap(),
        &args[2].downcast::<String>().unwrap()
    ));
    let empty = ConstructorMetadata::new("Empty", None);
    assert_eq!(
        service
            .create_instance(&empty, vec![ServiceValue::new(1)], |a| a.len())
            .unwrap(),
        1
    );
    let result: Result<(), &str> = service
        .create_instance(&empty, vec![], |_| Err("constructor failure"))
        .unwrap();
    assert_eq!(result, Err("constructor failure"));
}
