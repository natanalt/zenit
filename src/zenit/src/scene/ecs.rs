//! Zenit's custom-built ECS
//!
//! It's singlethreaded with thread-local globals. The implementation will need
//! reworking at some point. The usage will stay the same, at least.
//!

use smallvec::SmallVec;
use std::{
    any::{Any, TypeId},
    cell::{self, Cell, RefCell, UnsafeCell},
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use thiserror::Error;

thread_local! {
    static UNI: RefCell<Universe> = RefCell::new(Universe::new());
}

/// A universe is a container of entities.
///
/// Generally, you should only use the thread local universe, although it is
/// possible to use custom ones - just keep in mind that [`Entity`] callbacks
/// only work with that global universe.
pub struct Universe {
    entities: Vec<Option<Rc<EntityBox>>>,
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum EntityValidationError {
    #[error("invalid entity descriptor (index points to a nonexistent entity)")]
    BadIndex,
    #[error("invalid entity descriptor (bad generation; use after free?)")]
    GenerationTooLow,
    #[error("invalid entity descriptor (bad generation; above current)")]
    GenerationTooHigh,
}

impl Universe {
    /// Creates a blank universe
    pub const fn new() -> Self {
        Self { entities: vec![] }
    }

    /// Creates a blank entity and returns its descriptor.
    ///
    /// ## Panics
    /// Panics when the entity cannot be created. The most likely reason for
    /// that could be OOM.
    pub fn create_entity(&mut self) -> Entity {
        todo!()
    }

    /// Destroys specified entity.
    ///
    /// ## Panics
    /// If debug assertions are enabled, panics if the entity is invalid
    pub fn destroy_entity(&mut self, entity: Entity) {
        #[cfg(debug_assertions)]
        self.validate_entity(entity)
            .expect("attempted to destroy invalid entity");
        todo!()
    }

    /// Validates whether given entity descriptor points to a valid entity.
    #[inline]
    pub fn validate_entity(&self, entity: Entity) -> Result<(), EntityValidationError> {
        let ebox = self
            .entities
            .get(entity.index)
            .ok_or(EntityValidationError::BadIndex)?
            .as_ref()
            .ok_or(EntityValidationError::BadIndex)?;

        if entity.generation < ebox.generation {
            Err(EntityValidationError::GenerationTooLow)
        } else if entity.generation > ebox.generation {
            Err(EntityValidationError::GenerationTooHigh)
        } else {
            Ok(()) // ur valid :3
        }
    }

    fn get_entity_box(&self, entity: Entity) -> Result<Rc<EntityBox>, EntityValidationError> {
        self.validate_entity(entity)?;
        Ok(self
            .entities
            .get(entity.index)
            .unwrap()
            .as_ref()
            .unwrap()
            .clone())
    }
}

/// A cheap entity descriptor. It contains two pointer-sized fields, an index
/// to the entity, and its generation which is used to detect use-after-free
/// entity descriptors.
///
/// Functions within this struct operate on the thread-local-global universe,
/// for convenience.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub index: usize,
    pub generation: NonZeroUsize,
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum EntityBorrowError {
    #[error("entity is mutably borrowed")]
    AlreadyBorrowed,
    #[error(transparent)]
    ValidationError(#[from] EntityValidationError),
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum MutEntityBorrowError {
    #[error("entity is immutably borrowed")]
    AlreadyBorrowed,
    #[error(transparent)]
    ValidationError(#[from] EntityValidationError),
}

impl Entity {
    /// Creates a new entity, see [`Universe::create_entity`]
    #[inline]
    pub fn new() -> Self {
        UNI.with(|unicell| unicell.borrow_mut().create_entity())
    }

    /// Destroys this entity, invalidating this descriptor permanently.
    pub fn destroy(self) {
        UNI.with(|unicell| unicell.borrow_mut().destroy_entity(self));
    }

    /// Checks whether this descriptor points to a valid entity.
    pub fn validate_descriptor(self) -> Result<(), EntityValidationError> {
        UNI.with(|unicell| unicell.borrow().validate_entity(self))
    }

    /// Attempts to immutably borrow this entity.
    pub fn try_borrow(self) -> Result<EntityBorrow, EntityBorrowError> {
        UNI.with(|unicell| {
            let universe = unicell.borrow();
            let ebox = universe.get_entity_box(self)?;

            let borrow =
                EntityBorrow::try_borrow(ebox).ok_or(EntityBorrowError::AlreadyBorrowed)?;

            Ok(borrow)
        })
    }

    /// Attempts to immutably borrow this entity.
    ///
    /// ## Panics
    /// Panics if the entity cannot be borrowed, which may happen if the entity
    /// is currently mutably borrowed somewhere else.
    pub fn borrow(self) -> EntityBorrow {
        self.try_borrow().expect("couldn't borrow entity")
    }

    /// Attempts to mutably borrow this entity.
    pub fn try_borrow_mut(self) -> Result<MutEntityBorrow, MutEntityBorrowError> {
        UNI.with(|unicell| {
            let universe = unicell.borrow();
            let ebox = universe.get_entity_box(self)?;

            let borrow =
                MutEntityBorrow::try_borrow(ebox).ok_or(MutEntityBorrowError::AlreadyBorrowed)?;

            Ok(borrow)
        })
    }

    /// Attempts to mutably borrow this entity.
    ///
    /// ## Panics
    /// Panics if the entity cannot be borrowed, which may happen if the entity
    /// is currently borrowed somewhere else.
    pub fn borrow_mut(self) -> MutEntityBorrow {
        self.try_borrow_mut()
            .expect("couldn't mutably borrow entity")
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
/// IDs which are stored within entities.
pub trait Tag: Any {}

/// A component, attachable to an entity
pub trait Component: Any {
    /// Called every frame for every component
    fn process(&mut self, _parent: MutEntityBorrow) {}
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
    call_process: fn(Rc<RefCell<dyn Any>>, MutEntityBorrow),

    /// The component is held within an Rc to prevent borrow issues when calling
    /// process callbacks for each component. Consider this more of an
    /// implementation detail than anything else, or even a workaround/hack.
    ///
    // TODO: think of a better solution for this?
    //       perhaps wrapping parent and children within dedicated cells and
    //       only ever exposing entities immutably, allowing for a double
    //       borrow?
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

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum ComponentBorrowError {
    #[error("component not found")]
    NotFound,
    #[error("the component is already borrowed")]
    AlreadyBorrowed,
}

/// The internals of an entity, accessible via a borrow from the universe.
pub struct EntityData {
    location: Entity,
    parent: Option<Entity>,
    children: SmallVec<[Entity; 5]>,
    components: HashMap<TypeId, HeldComponent>,
    tags: HashSet<TypeId>,
}

impl EntityData {
    /// Creates a new [`EntityData`] instance. Generally shouldn't be called.
    pub fn new(location: Entity) -> Self {
        Self {
            location,
            parent: None,
            children: SmallVec::new(),
            components: HashMap::new(),
            tags: HashSet::new(),
        }
    }

    /// Adds a specified tag to this entity. If it's already there, nothing
    /// happens. Returns previous state of the tag.
    pub fn add_tag<T: Tag>(&mut self) -> bool {
        self.tags.insert(TypeId::of::<T>())
    }

    /// Removes specified tag from this entity. If it wasn't there, nothing
    /// happens. Returns previous state of the tag.
    pub fn remove_tag<T: Tag>(&mut self) -> bool {
        self.tags.remove(&TypeId::of::<T>())
    }

    /// Returns whether this entity has a specified tag
    pub fn has_tag<T: Tag>(&self) -> bool {
        self.tags.contains(&TypeId::of::<T>())
    }

    /// Adds a specified component to this entity. If it already exists, it
    /// will be overwritten.
    pub fn add_component(&mut self, c: impl Component) {
        self.components.insert(c.type_id(), HeldComponent::new(c));
    }

    /// Adds a component to this entity, using its [`Default`] implementation.
    /// If it already exists, it will be overwritten.
    pub fn make_component<C: Component + Default>(&mut self) {
        self.add_component(C::default());
    }

    /// Attempts to lock immutable access to given component, returning with a
    /// [`ComponentBorrowError`] if not possible.
    pub fn try_get_component<C: Component>(&self) -> Result<cell::Ref<C>, ComponentBorrowError> {
        let borrow = self
            .components
            .get(&TypeId::of::<C>())
            .ok_or(ComponentBorrowError::NotFound)?
            .component
            .try_borrow()
            .map_err(|_| ComponentBorrowError::AlreadyBorrowed)?;

        Ok(cell::Ref::map(borrow, |value| {
            value.downcast_ref().expect("invalid component type")
        }))
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
        &mut self,
    ) -> Result<cell::RefMut<C>, ComponentBorrowError> {
        let borrow = self
            .components
            .get(&TypeId::of::<C>())
            .ok_or(ComponentBorrowError::NotFound)?
            .component
            .try_borrow_mut()
            .map_err(|_| ComponentBorrowError::AlreadyBorrowed)?;

        Ok(cell::RefMut::map(borrow, |value| {
            value.downcast_mut().expect("invalid component type")
        }))
    }

    /// Attempts to lock mutable access to given component.
    ///
    /// ## Panics
    /// Panics if the component is already borrowed.
    pub fn get_component_mut<C: Component>(&mut self) -> cell::RefMut<C> {
        self.try_get_component_mut()
            .expect("unable to borrow component")
    }

    /// Iterates this entity's children.
    pub fn iter_children(&self) -> impl Iterator<Item = Entity> + '_ {
        self.children.iter().map(|e| *e)
    }

    /// Adds a child to this entity. The child must not have a parent.
    ///
    /// ## Panics
    /// Panics if the child has a parent.
    pub fn add_child(&mut self, entity: &mut MutEntityBorrow) {
        assert!(entity.parent.is_none(), "the child must be an orphan");
        entity.parent = Some(self.location);
        self.children.push(entity.location);
    }

    /// Removes a child from this entity. Self must be a parent of passed
    /// child entity.
    ///
    /// ## Panics
    /// Panics if the self entity is the child's parent
    pub fn remove_child(&mut self, entity: &mut MutEntityBorrow) {
        assert!(
            entity.parent == Some(self.location),
            "the child's parent must be self entity"
        );

        entity.parent = None;
        self.children.retain(|child| *child != entity.location);
    }
}

struct EntityBox {
    generation: NonZeroUsize,
    data: UnsafeCell<EntityData>,
    flag: Cell<i32>,
}

impl EntityBox {
    #[inline]
    pub fn is_idle(&self) -> bool {
        self.flag.get() == 0
    }

    #[inline]
    pub fn is_immutably_borrowed(&self) -> bool {
        self.flag.get() > 0
    }

    #[inline]
    pub fn is_mutably_borrowed(&self) -> bool {
        self.flag.get() < 0
    }

    #[inline]
    pub fn add_immutable_borrow(&self) {
        debug_assert!(self.is_idle() || self.is_immutably_borrowed());
        self.update_borrow(|flag| flag + 1);
    }

    #[inline]
    pub fn remove_immutable_borrow(&self) {
        debug_assert!(self.is_immutably_borrowed());
        self.update_borrow(|flag| flag - 1);
    }

    #[inline]
    pub fn add_mutable_borrow(&self) {
        debug_assert!(self.is_idle());
        self.update_borrow(|flag| flag - 1);
    }

    #[inline]
    pub fn remove_mutable_borrow(&self) {
        debug_assert!(self.is_mutably_borrowed());
        self.update_borrow(|flag| flag + 1);
    }

    #[inline]
    fn update_borrow(&self, f: impl FnOnce(i32) -> i32) {
        self.flag.set(f(self.flag.get()));
    }
}

pub struct EntityBorrow(Rc<EntityBox>);

impl EntityBorrow {
    fn try_borrow(ebox: Rc<EntityBox>) -> Option<Self> {
        if ebox.is_idle() || ebox.is_mutably_borrowed() {
            ebox.add_immutable_borrow();
            Some(Self(ebox))
        } else {
            None
        }
    }
}

impl Deref for EntityBorrow {
    type Target = EntityData;
    fn deref(&self) -> &EntityData {
        // Safety: aliasing rules preserved by EntityBorrow structs
        unsafe { self.0.data.get().as_ref().unwrap() }
    }
}

impl Drop for EntityBorrow {
    fn drop(&mut self) {
        self.0.remove_immutable_borrow();
    }
}

pub struct MutEntityBorrow(Rc<EntityBox>);

impl MutEntityBorrow {
    fn try_borrow(ebox: Rc<EntityBox>) -> Option<Self> {
        if ebox.is_idle() {
            ebox.add_mutable_borrow();
            Some(Self(ebox))
        } else {
            None
        }
    }
}

impl Deref for MutEntityBorrow {
    type Target = EntityData;
    fn deref(&self) -> &EntityData {
        // Safety: aliasing rules preserved by EntityBorrow structs
        unsafe { self.0.data.get().as_ref().unwrap() }
    }
}

impl DerefMut for MutEntityBorrow {
    fn deref_mut(&mut self) -> &mut EntityData {
        // Safety: aliasing rules preserved by EntityBorrow structs
        unsafe { self.0.data.get().as_mut().unwrap() }
    }
}

impl Drop for MutEntityBorrow {
    fn drop(&mut self) {
        self.0.remove_mutable_borrow();
    }
}
