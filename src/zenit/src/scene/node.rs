//! Zenit's custom-built, singlethreaded (because yes) ECS
//!

use glam::Affine3A;
use smallvec::SmallVec;
use std::{
    any::{Any, TypeId},
    cell::{self, RefCell},
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
};
use thiserror::Error;

/// A node reference. Can be cheaply copied and passed around. Can also
/// be converted into [`NodeWeak`].
///
/// ## Typing
/// Each node comes with a generic parameter `T` that must implement [`NodeImpl`].
/// By default it's `dyn NodeImpl`, in which case the node is completely untyped.
///
/// It is possible to convert between typed and untyped versions of the node by
/// using the `cast` and `into_untyped` functions.
#[derive(Clone)]
pub struct NodeRef<T: Node + ?Sized = dyn Node>(pub Rc<NodeInner<T>>);

/// Common node functions
impl<T: Node + ?Sized> NodeRef<T> {}

/// Untyped node functions
impl NodeRef<dyn Node> {
    /// Attempts to cast this untyped node into a typed one. Returns [`None`]
    /// if the node type doesn't match the target type.
    pub fn cast<T: Node>(self) -> Option<NodeRef<T>> {
        let rc = self.0;

        if rc.imp_type == TypeId::of::<T>() {
            // Safety: we just verified type equivalence
            let casted = unsafe { Rc::from_raw(Rc::into_raw(rc) as *const NodeInner<T>) };
            Some(NodeRef(casted))
        } else {
            None
        }
    }
}

/// Typed node functions
impl<T: Node> NodeRef<T> {
    /// Directly creates a new node out
    pub fn new(value: T) -> NodeRef<T> {
        NodeRef(Rc::new(NodeInner {
            data: RefCell::new(NodeData::default()),
            imp_type: TypeId::of::<T>(),
            imp: RefCell::new(value),
        }))
    }

    /// Converts this typed node reference into an untyped one.
    pub fn into_untyped(self) -> NodeRef {
        NodeRef(self.0 as Rc<NodeInner<dyn Node>>)
    }
}

#[derive(Clone)]
pub struct NodeWeak<T: Node + ?Sized = dyn Node>(Weak<NodeInner<T>>);

impl<T: Node + ?Sized> NodeWeak<T> {
    /// Attempts to convert this weak node refrerence into a strong one.
    pub fn upgrade(&self) -> Option<NodeRef<T>> {
        self.0.upgrade().map(NodeRef)
    }
}

/// Modifiable node details
#[derive(Default)]
pub struct NodeData {
    /// Node label, used mainly for debugging purposes.
    pub label: Option<String>,
    parent: Option<NodeWeak>,
    tags: HashSet<TypeId>,

    /// Global transform, made out of the parent's global transform and this
    /// node's local transform.
    /// 
    /// If this node doesn't have a parent, then it's the same as local
    /// transform.
    /// 
    /// Must be recalculated whenever the parent's transform or this local
    /// transform gets updated.
    global_transform: Affine3A,
    local_transform: Affine3A,

    // TODO: figure out whether children could be stored in a sorted way to make lookups faster
    children: SmallVec<[NodeRef; 4]>,
}

impl NodeData {
    /// Returns this node's local transform.
    pub fn local_transform(&self) -> Affine3A {
        self.local_transform
    }

    /// Returns this node's global transform.
    pub fn global_transform(&self) -> Affine3A {
        self.global_transform
    }

    /// Updates this node's local transform, additionally causing recalculation
    /// of the global transform. If this node has children, this operation will
    /// also cause recursive recalculations of their global transforms.
    /// 
    /// ## Panics
    /// 
    pub fn set_local_transform(&mut self, lt: Affine3A) {
        self.local_transform = lt;
        self.global_transform = match self.parent() {
            Some(parent) => todo!(),
            None => lt,
        }
    }

    /// Adds a tag to this node. Returns previous tag state.
    pub fn add_tag<T: Tag>(&mut self) -> bool {
        self.tags.insert(TypeId::of::<T>())
    }

    /// Removes a tag from this node. Returns previous state.
    pub fn remove_tag<T: Tag>(&mut self) -> bool {
        self.tags.remove(&TypeId::of::<T>())
    }

    /// Returns whether this node has specified tag
    pub fn has_tag<T: Tag>(&self) -> bool {
        self.tags.contains(&TypeId::of::<T>())
    }

    /// Checks whether this child has a living parent.
    pub fn has_parent(&self) -> bool {
        self.parent
            .as_ref()
            .map(|weak| weak.0.strong_count() > 0)
            .unwrap_or(false)
    }

    /// Returns this node's parent. If there's no parent, or the parent has
    /// been destroyed, [`None`] is returned instead.
    pub fn parent(&self) -> Option<NodeRef> {
        self.parent.as_ref().and_then(NodeWeak::upgrade)
    }

    /// Checks whether given node is a child of this node
    pub fn has_child(&self, node: &NodeRef) -> bool {
        // there probably is some fancy iterator thing to make this
        // :sparkles: functional :sparkles: and stuff idk
        for child in &self.children {
            if Rc::ptr_eq(&node.0, &child.0) {
                return true;
            }
        }
        false
    }

    /// Adds a new child to this node. This node must not have a parent, nor
    /// already be a child.
    ///
    /// Currently there are debug-only checks for that.
    ///
    /// ## Panics
    /// Panics if any of the child debug checks fails or if the node's data
    /// cannot be locked as mutable
    pub fn add_child(&mut self, child_node: NodeRef) {
        debug_assert!(!self.has_child(&child_node), "the node must not be already a child");

        let mut child = child_node
            .0
            .data
            .try_borrow_mut()
            .expect("new child node is locked");
        
        debug_assert!(child.parent().is_none(), "the child must not have a parent");

        todo!()
    }
}

/// Node functionality.
pub trait Node: Any {
    /// Called every frame.
    fn process(&mut self, _data: &mut NodeData) {}
}

/// Marker trait for node tags.
pub trait Tag: Any {}

/// Internal implementation details of a node
pub struct NodeInner<T: Node + ?Sized> {
    data: RefCell<NodeData>,
    imp_type: TypeId,
    imp: RefCell<T>,
}
