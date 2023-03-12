//! ECS-defined macros

/// Creates the [`Universe`] type. It generates all the code necessary for managing components in a
/// form that's slightly more convenient than a hash map of dynamic vec-like objects.
/// 
/// It should **only** be used in `ecs/universe.rs`, this thing doesn't even handle imports properly.
#[macro_export]
macro_rules! create_universe {
    (
        components {
            $($type:ty),* $(,)*
        }
    ) => {
        paste! {
            /// Trait marking types that were registered during creation of the [`Universe`] type.
            /// Its purpose is to provide compile-time checking of component accesses, to ensure
            /// that a component wasn't forgotten to be registered in `src/ecs/mod.rs`.
            ///
            /// Code outside the macro must not implement it. Doing so will cause runtime crashes
            /// in component related universe functions, which won't be able to handle the illegally
            /// registered type.
            pub trait RegisteredComponent: Component {}
            $(impl RegisteredComponent for $type {})*

            #[allow(non_snake_case)] // we're generating names from type names
            pub struct Universe {
                top_generation: NonZeroU32,
                free_indices: Vec<u32>,

                generations: Vec<Option<NonZeroU32>>,

                $([<$type _values>]: ComponentVec<$type>,)*
            }

            impl Universe {
                /// Creates a blank, empty universe.
                pub fn new() -> Self {
                    Self {
                        top_generation: NonZeroU32::new(1).unwrap(),
                        free_indices: vec![],
                        generations: vec![],
                        $([<$type _values>]: ComponentVec::new(),)*
                    }
                }

                /// Returns an immutable reference to the specified component vector.
                #[inline(always)] // This should optimize out to a simple pointer calculation
                pub(super) fn get_component_vec<T: RegisteredComponent>(&self) -> &ComponentVec<T> {
                    let tid = TypeId::of::<T>();

                    $(
                        if tid == TypeId::of::<$type>() {
                            // Safety: We just verified the type matches.
                            //         On the type level this is literally a no-op.
                            unsafe {
                                return mem::transmute(&self.[<$type _values>]);
                            }
                        }
                    )*

                    // This *shouldn't happen* unless someone manually implements RegisteredComponent
                    // which would be a mistake
                    unreachable!("unregistered component access");
                }

                /// Returns mutable references to the component vector, and the generation vector,
                /// accesss to which would otherwise be blocked by the borrow checker.
                #[inline(always)] // This should optimize out to a simple pointer calculation
                pub(super) fn get_component_vec_mut<T: RegisteredComponent>(&mut self) -> (
                    &mut ComponentVec<T>,
                    &mut Vec<Option<NonZeroU32>>
                ) {
                    let tid = TypeId::of::<T>();

                    $(
                        if tid == TypeId::of::<$type>() {
                            // Safety: see get_component_vec
                            unsafe {
                                return (
                                    mem::transmute(&mut self.[<$type _values>]),
                                    &mut self.generations,
                                );
                            }
                        }
                    )*

                    unreachable!("unregistered component access");
                }

                pub fn delete_entity(&mut self, entity: Entity) {
                    assert!(
                        self.validate_entity(entity),
                        "attempting to delete an invalid entity"
                    );

                    self.generations[entity.index as usize] = None;

                    $(
                        self.[<$type _values>].clear(entity.index);
                    )*

                    self.free_indices.push(entity.index);
                }
            }
        }
    };
}
