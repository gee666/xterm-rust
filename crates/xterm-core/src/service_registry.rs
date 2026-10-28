// Copyright (c) 2019 The xterm.js authors. MIT license.
// Inspired by Microsoft Corporation's MIT-licensed dependency injection system.
//! Explicit, typed equivalents of xterm's service decorators.
//!
//! A registry is owned rather than process-global. Share it with each terminal's
//! constructors to preserve decorator identity without global mutable state.
use std::any::TypeId;
use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Debug)]
struct Identity {
    name: String,
    kind: TypeId,
}

/// Untyped identity for constructor dependency metadata.
#[derive(Clone, Debug)]
pub struct ServiceKey(Rc<Identity>);
impl ServiceKey {
    pub fn name(&self) -> &str {
        &self.0.name
    }
    pub(crate) fn kind(&self) -> TypeId {
        self.0.kind
    }
}
impl PartialEq for ServiceKey {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ServiceKey {}

/// A service's identity and expected Rust value type.
#[derive(Debug)]
pub struct ServiceId<T: 'static> {
    key: ServiceKey,
    marker: PhantomData<fn() -> T>,
}
impl<T> Clone for ServiceId<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            marker: PhantomData,
        }
    }
}
impl<T> ServiceId<T> {
    pub fn key(&self) -> ServiceKey {
        self.key.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    TypeMismatch(String),
    DecoratorArity,
}
impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeMismatch(name) => write!(
                f,
                "service {name} was registered with a different Rust type"
            ),
            Self::DecoratorArity => {
                f.write_str("@IServiceName-decorator can only be used to decorate a parameter")
            }
        }
    }
}
impl std::error::Error for RegistryError {}

#[derive(Default, Debug)]
pub struct ServiceRegistry {
    identities: Vec<ServiceKey>,
}
impl ServiceRegistry {
    /// Intern a name. Rust rejects incompatible generic requests for an existing name.
    pub fn create_decorator<T: 'static>(
        &mut self,
        name: &str,
    ) -> Result<ServiceId<T>, RegistryError> {
        let key = if let Some(key) = self.identities.iter().find(|key| key.name() == name) {
            if key.kind() != TypeId::of::<T>() {
                return Err(RegistryError::TypeMismatch(name.to_owned()));
            }
            key.clone()
        } else {
            let key = ServiceKey(Rc::new(Identity {
                name: name.to_owned(),
                kind: TypeId::of::<T>(),
            }));
            self.identities.push(key.clone());
            key
        };
        Ok(ServiceId {
            key,
            marker: PhantomData,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub id: ServiceKey,
    pub index: usize,
}

/// Shared dependency array returned by [`ConstructorMetadata::dependencies_view`].
/// Drop borrowed guards before calling metadata methods or constructing instances.
pub type DependencyList = Rc<RefCell<Vec<Dependency>>>;

/// Metadata inheritance matches JavaScript constructor property inheritance.
/// The first own decorator replaces inherited dependencies rather than appending.
#[derive(Clone, Debug)]
pub struct ConstructorMetadata(Rc<RefCell<Metadata>>);
#[derive(Debug)]
struct Metadata {
    name: String,
    parent: Option<ConstructorMetadata>,
    own: Option<DependencyList>,
}
impl ConstructorMetadata {
    pub fn new(name: impl Into<String>, parent: Option<Self>) -> Self {
        Self(Rc::new(RefCell::new(Metadata {
            name: name.into(),
            parent,
            own: None,
        })))
    }
    pub fn name(&self) -> String {
        self.0.borrow().name.clone()
    }
    /// Arity is explicit because Rust does not expose JavaScript decorators.
    pub fn decorate(
        &self,
        id: ServiceKey,
        index: usize,
        arity: usize,
    ) -> Result<(), RegistryError> {
        if arity != 3 {
            return Err(RegistryError::DecoratorArity);
        }
        let own = self
            .0
            .borrow_mut()
            .own
            .get_or_insert_with(|| Rc::new(RefCell::new(Vec::new())))
            .clone();
        own.borrow_mut().push(Dependency { id, index });
        Ok(())
    }
    /// Resolve the source's live dependency array. Inherited annotations share
    /// their owner's array until the child receives its first own decorator.
    /// With no annotations, each call returns a fresh, unattached empty array.
    pub fn dependencies_view(&self) -> DependencyList {
        let state = self.0.borrow();
        match &state.own {
            Some(own) => Rc::clone(own),
            None => state
                .parent
                .as_ref()
                .map(Self::dependencies_view)
                .unwrap_or_else(|| Rc::new(RefCell::new(Vec::new()))),
        }
    }
    /// Owned snapshot for callers that do not need live mutation or identity.
    pub fn dependencies(&self) -> Vec<Dependency> {
        self.dependencies_view().borrow().clone()
    }
    pub(crate) fn sorted_dependencies(&self) -> Vec<Dependency> {
        let dependencies = self.dependencies_view();
        let mut dependencies = dependencies.borrow_mut();
        dependencies.sort_by_key(|dependency| dependency.index);
        dependencies.clone()
    }
}
