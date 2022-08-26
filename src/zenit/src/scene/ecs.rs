//! Zenit's custom-built, singlethreaded (because yes) ECS
//!

use smallvec::SmallVec;
use std::{
    any::{Any, TypeId},
    cell::{self, RefCell},
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
};
use thiserror::Error;

/// Reference-counted handle to an entity. Backed by an [`Rc`], it is very cheap
/// to copy via [`Clone`], and is pointer-sized, with `Option<Entity>` size
/// optimizations available.
#[derive(Clone)]
pub struct Entity(Rc<EntityData>);

impl Eq for Entity {}
impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Entity {
    /// Creates a new blank entity.
    pub fn new() -> Self {
        Self(Rc::new(EntityData::default()))
    }

    /// Adds a new component into this entity. If it already exists, it is
    /// replaced.
    ///
    /// ## Panics
    /// Panics if the component is present and locked (for example when process
    /// is being called.)
    pub fn add_component(&self, c: impl Component) {
        todo!()
    }

    /// Removes specified component from this entity. Does nothing if it doesn't
    /// already exist.
    ///
    /// ## Panics
    /// Panics if the component is present and locked (for example when process
    /// is being called.)
    pub fn remove_component<C: Component>(&self) {
        todo!()
    }

    /// Checks if specified component exists.
    pub fn has_component<C: Component>(&self) -> bool {
        todo!()
    }

    /// Adds a component to this entity, using its [`Default`] implementation.
    /// If it already exists, it will be overwritten.
    ///
    /// ## Panics
    /// Panics if the component is present and locked (for example when process
    /// is being called.)
    pub fn make_component<C: Component + Default>(&self) {
        self.add_component(C::default());
    }

    /// Attempts to lock immutable access to given component, returning with a
    /// [`ComponentBorrowError`] if not possible.
    pub fn try_get_component<C: Component>(&self) -> Result<cell::Ref<C>, ComponentBorrowError> {
        todo!()
    }

    /// Attempts to lock immutable access to given component.
    ///
    /// ## Panics
    /// Panics if the component cannot be borrowed.
    pub fn get_component<C: Component>(&self) -> cell::Ref<C> {
        self.try_get_component()
            .expect("unable to borrow component")
    }

    /// Attempts to lock mutable access to given component, returning with a
    /// [`ComponentBorrowError`] if not possible.
    pub fn try_get_component_mut<C: Component>(
        &self,
    ) -> Result<cell::RefMut<C>, ComponentBorrowError> {
        todo!()
    }

    /// Attempts to lock mutable access to given component.
    ///
    /// ## Panics
    /// Panics if the component is already borrowed.
    pub fn get_component_mut<C: Component>(&self) -> cell::RefMut<C> {
        self.try_get_component_mut()
            .expect("unable to borrow component")
    }

    /// Adds the tag to this entity. Returns the tag's previous state.
    pub fn add_tag<T: Tag>(&self) -> bool {
        self.0.tags.borrow_mut().insert(TypeId::of::<T>())
    }

    /// Removes the tag from this entity. Returns the tag's previous state.
    pub fn remove_tag<T: Tag>(&self) -> bool {
        self.0.tags.borrow_mut().remove(&TypeId::of::<T>())
    }

    /// Checks if the entity has specified tag.
    pub fn has_tag<T: Tag>(&self) -> bool {
        self.0.tags.borrow().contains(&TypeId::of::<T>())
    }

    /// Returns an iterator over this entity's children. For the lifetime of
    /// this iterator, the child list becomes locked and unmodifiable.
    ///
    /// Multiple iterators may be alive at the same time.
    ///
    /// ## Panics
    ///  * if the child list is already borrowed for some reason
    pub fn iter_children<'a>(&'a self) -> impl Iterator<Item = Entity> + 'a {
        struct ChildIterator<'a> {
            borrow: cell::Ref<'a, SmallVec<[Entity; 5]>>,
            index: usize,
        }

        impl<'a> Iterator for ChildIterator<'a> {
            type Item = Entity;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.index < self.borrow.len() {
                    self.index += 1;
                    Some(self.borrow[self.index - 1].clone())
                } else {
                    None
                }
            }
        }

        ChildIterator::<'a> {
            borrow: self.0.children.borrow(),
            index: 0,
        }
    }

    /// Adds a new child to this entity.
    ///
    /// ## Panics
    ///  * if the child has a parent
    ///  * if the child list is locked (for example by `iter_children`)
    pub fn add_child(&self, child: &Entity) {
        let mut parent = child.0.parent.borrow_mut();
        assert!(parent.is_none(), "the child must be an orphan");
        *parent = Some(self.as_weak());
        self.0.children.borrow_mut().push(child.clone());
    }

    /// Removes a child from this entity.
    ///
    /// ## Panics
    ///  * if the child's parent isn't this entity
    ///  * if the child list is locked (for example by `iter_children`)
    pub fn remove_child(&self, child: &Entity) {
        let mut parent = child.0.parent.borrow_mut();
        assert!(
            *parent == Some(self.as_weak()),
            "this entity must be the child's parent"
        );
        *parent = None;
        self.0.children.borrow_mut().retain(|other| child != other);
    }

    /// Calls the process callback of specified [`Component`]. This generally
    /// should only be called by scene loop code.
    ///
    /// ## Panics
    ///  * if the component doesn't exist
    ///  * if the component cannot be mutably locked
    pub fn call_process_for<C: Component>(&self) {
        let components = self.0.components.borrow();
        let held = components
            .get(&TypeId::of::<C>())
            .expect("component doesn't exist");
        (held.call_process)(held.component.clone(), self.clone());
    }

    /// Returns the parent, if there's one.
    ///
    /// ## Storage details
    /// Each entity stores a weak reference to a parent. If the parent is
    /// deallocated, this function returns [`None`], without updating
    /// the internal value.
    pub fn parent(&self) -> Option<Entity> {
        self.0
            .parent
            .borrow()
            .clone()
            .map(EntityWeak::upgrade)
            .flatten()
    }

    /// Creates a new weak reference, consuming this one.
    pub fn into_weak(self) -> EntityWeak {
        self.into()
    }

    /// Creates a new weak reference without consuming this one.
    pub fn as_weak(&self) -> EntityWeak {
        self.clone().into()
    }
}

impl Into<EntityWeak> for Entity {
    fn into(self) -> EntityWeak {
        EntityWeak(Rc::downgrade(&self.0))
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum ComponentBorrowError {
    #[error("component not found")]
    NotFound,
    #[error("the component is already borrowed")]
    AlreadyBorrowed,
}

/// Weak reference version of [`Entity`]. What else can be said, see docs of
/// [`Weak`] for details.
#[derive(Clone)]
pub struct EntityWeak(Weak<EntityData>);

impl Eq for EntityWeak {}
impl PartialEq for EntityWeak {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

impl EntityWeak {
    /// Attempts to convert this weak reference to a strong one. Returns None
    /// if the entity doesn't exist anymore, making this conversion impossible.
    pub fn upgrade(self) -> Option<Entity> {
        self.0.upgrade().map(|rc| Entity(rc))
    }
}

// TODO: figure out a way to store tags in a bitmap or something?
//       Instead of holding tags in a HashSet, which requires heap allocations
//       and so on, tags could potentially fit within, like, a single u64 of u128.
//
//       I see two ways of approaching this:
//        - somehow automatically picking up all Tag types and assigning them bit
//          indices at compile time
//        - assigned said bit indices at runtime, which would probably be even slower
//          than the current approach
/// Marker trait for a tag. Tags aren't used for anything besides for their type
/// IDs which are stored within entities, cheap to query and modify.
pub trait Tag: Any {}

/// A component, attachable to an entity. Contains a process callback, called
/// every frame.
pub trait Component: Any {
    /// Called every frame for every component of every entity.
    fn process(&mut self, _parent: Entity) {}
}

/// The internals of an entity, accessed by [`Entity`] functions.
#[derive(Default)]
pub struct EntityData {
    parent: RefCell<Option<EntityWeak>>,
    children: RefCell<SmallVec<[Entity; 5]>>,
    components: RefCell<HashMap<TypeId, HeldComponent>>,
    tags: RefCell<HashSet<TypeId>>,
}

struct HeldComponent {
    /// A hacky workaround around Rust's type system, since holding the
    /// component as an `dyn Component` makes us unable to directly use
    /// downcast functions (without trait upcasting which is experimental
    /// at the time of writing).
    ///
    /// This function, created in the generic constructor of HeldComponent,
    /// calls Component::process by downcasting it internally.
    ///
    /// Also it's Rc as a hack, see HeldComponent::call_process
    call_process: fn(Rc<RefCell<dyn Any>>, Entity),

    /// The component is held within an Rc to prevent borrow issues when calling
    /// process callbacks for each component. Consider this more of an
    /// implementation detail than anything else, or even a workaround/hack.
    component: Rc<RefCell<dyn Any>>,
}

impl HeldComponent {
    pub fn new<C: Component>(c: C) -> Self {
        Self {
            call_process: |reference, entity_borrow| {
                let mut borrow = reference.borrow_mut();
                borrow
                    .downcast_mut::<C>()
                    .expect("invalid type in component map")
                    .process(entity_borrow);
            },
            component: Rc::new(RefCell::new(c)),
        }
    }
}
