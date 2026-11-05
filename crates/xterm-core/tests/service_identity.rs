use std::any::TypeId;
use std::rc::Rc;
use xterm_core::instantiation::{InstantiationService, ServiceCollection, ServiceValue};
use xterm_core::service_registry::{ConstructorMetadata, RegistryError, ServiceRegistry};

type General = fn(&());
type Static = fn(&'static ());

fn ignore(_: &()) {}

#[test]
fn function_pointer_lifetimes_remain_distinct_service_types() {
    assert_ne!(TypeId::of::<General>(), TypeId::of::<Static>());
    let mut registry = ServiceRegistry::default();
    let general = registry.create_decorator::<General>("callback").unwrap();
    let alias = general.clone();
    assert_eq!(general.key(), alias.key());
    assert!(matches!(
        registry.create_decorator::<Static>("callback"),
        Err(RegistryError::TypeMismatch(name)) if name == "callback"
    ));

    let collection = ServiceCollection::default();
    let callback: Rc<General> = Rc::new(ignore);
    collection.set(&general, Rc::clone(&callback));
    assert!(Rc::ptr_eq(&collection.get(&alias).unwrap(), &callback));
    let narrowed: Static = ignore;
    assert!(matches!(
        collection.set_value(general.key(), ServiceValue::new(narrowed)),
        Err(RegistryError::TypeMismatch(name)) if name == "callback"
    ));
    assert!(Rc::ptr_eq(&collection.get(&general).unwrap(), &callback));
}

#[test]
fn constructor_injects_each_function_type_with_its_original_identity() {
    let mut registry = ServiceRegistry::default();
    let general = registry.create_decorator::<General>("general").unwrap();
    let narrowed = registry.create_decorator::<Static>("static").unwrap();
    let service = InstantiationService::new(&mut registry).unwrap();
    let general_value: Rc<General> = Rc::new(ignore);
    let static_value: Rc<Static> = Rc::new(ignore);
    service.set_service(&general, Rc::clone(&general_value));
    service.set_service(&narrowed, Rc::clone(&static_value));
    let metadata = ConstructorMetadata::new("Callbacks", None);
    metadata.decorate(general.key(), 0, 3).unwrap();
    metadata.decorate(narrowed.key(), 1, 3).unwrap();
    service
        .create_instance(&metadata, vec![], |args| {
            assert_eq!(args.len(), 2);
            assert!(Rc::ptr_eq(
                &args[0].downcast::<General>().unwrap(),
                &general_value
            ));
            assert!(Rc::ptr_eq(
                &args[1].downcast::<Static>().unwrap(),
                &static_value
            ));
            assert!(args[0].downcast::<Static>().is_none());
            assert!(args[1].downcast::<General>().is_none());
        })
        .unwrap();
}
