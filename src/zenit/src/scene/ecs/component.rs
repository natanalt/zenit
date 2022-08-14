use super::Entity;
use std::{
    alloc::Layout,
    any::{Any, TypeId},
    collections::HashMap,
    num::NonZeroUsize,
};

pub trait Component: Any {
    fn process(&mut self, parent: &Entity);
}

pub struct Components {
    holders: HashMap<TypeId, Box<dyn ComponentHolderTrait>>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            holders: HashMap::new(),
        }
    }

    //pub fn of_type<T>(&mut self) -> impl Iterator<Item = &mut T>
    //where
    //    T: Component
    //{
    //    todo!();
    //    std::iter::once(todo!());
    //}
}

pub struct ComponentHolder<T: Component> {
    meta: ComponentMeta,
    generations: Vec<Option<NonZeroUsize>>,
    components: Vec<Option<T>>,
}

pub trait ComponentHolderTrait: Any {
    fn process_entity(&mut self, entity: &Entity);
}

impl<T: Component> ComponentHolderTrait for ComponentHolder<T> {
    fn process_entity(&mut self, entity: &Entity) {
        //self.components[entity.index].unwrap().process(entity);
        todo!()
    }
}

pub struct ComponentMeta {
    pub layout: Layout,
    process_caller: Box<dyn Fn(&mut [u8], &Entity)>,
}

impl ComponentMeta {
    pub fn new<T: Component>() -> Self {
        Self {
            layout: Layout::new::<T>(),
            process_caller: Box::new(|raw, parent| {
                // Verify size and alignment
                let layout = Layout::new::<T>();
                debug_assert_eq!(raw.len(), layout.size(), "missized raw component");
                debug_assert!(
                    (raw.as_ptr() as usize) % layout.align() == 0,
                    "misaligned raw component"
                );

                // Safety: caller must provide a valid object; size and alignment
                //         is validated here
                let casted_pointer = raw.as_mut_ptr() as *mut T;
                let component = unsafe { casted_pointer.as_mut().unwrap() };
                component.process(parent);
            }),
        }
    }

    /// Caller ensures that given array is a valid object for this ComponentMeta
    /// object. Size and alignment are verified in the caller.
    pub unsafe fn call_process(&self, raw: &mut [u8], parent: &Entity) {
        (self.process_caller)(raw, parent);
    }
}
