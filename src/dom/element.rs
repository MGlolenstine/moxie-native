use super::{EventHandler, Node};
use crate::layout::{LayoutOptions, LayoutType};
use crate::render::PaintDetails;
use std::borrow::Cow;

/// Represents the attributes and behavior of a single DOM element.
pub trait Element: Default + Clone + PartialEq + 'static {
    /// The type of children that can be parented to this element.
    type Child: NodeChild + Clone + PartialEq;

    /// Attributes passed in from the element builder, to be decoded and set on this element.
    fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>);

    /// Describes how this element should be laid out.
    fn create_layout_opts(&self, parent_opts: &LayoutOptions) -> LayoutOptions;

    /// Describes how this element should be displayed on the screen.
    /// Return None for this element to only affect layout.
    fn paint(&self) -> Option<PaintDetails> {
        None
    }
}

/// The trait representing all events that can be invoked on an element.
pub trait Event {}

/// Statically defines the relationship between which elements can have
/// which events listened to, and also provides the mechanism for that
/// to happen via the set_handler method.
pub trait CanSetEvent<Ev>
where
    Ev: Event,
{
    fn set_handler(&mut self, handler: EventHandler<Ev>);
}

/// Because some elements need to have multiple types of elements
/// parented to them, their `Element::Child` type is actually an enum
/// (defined using the `multiple_children!` macro).
///
/// This trait abstracts over the children of an element so that these
/// enums don't have to implement Element directly. This trait provides
/// a sort of visitor pattern which lets the DOM be walked without
/// having to know the types of each element at each step.
pub trait NodeChild: 'static {
    /// Typically a pass-through to `Element::paint()`.
    fn paint(&self) -> Option<PaintDetails>;
    /// Typically a pass-through to `Element::create_layout_opts()`.
    fn create_layout_opts(&self, parent_opts: &LayoutOptions) -> LayoutOptions;
    /// Returns a trait object for the child at the given index. If the
    /// index is out of bounds, return None. Typically maps to
    /// `Element::children().get(index)`.
    fn get_child(&self, child: usize) -> Option<&dyn NodeChild>;
}

/// A helper to walk through the children of a `NodeChild`, creating an
/// iterator over the children so that you don't have to call
/// `get_child` manually.
pub fn children(node: &dyn NodeChild) -> impl Iterator<Item = &dyn NodeChild> {
    struct Iter<'a> {
        node: &'a dyn NodeChild,
        index: usize,
    }

    impl<'a> Iterator for Iter<'a> {
        type Item = &'a dyn NodeChild;

        fn next(&mut self) -> Option<Self::Item> {
            let child = self.node.get_child(self.index);
            self.index += 1;
            child
        }
    }

    Iter {
        node: node,
        index: 0,
    }
}

impl<Elt> NodeChild for Node<Elt>
where
    Elt: Element,
{
    fn paint(&self) -> Option<PaintDetails> {
        Element::paint(self.element())
    }

    fn create_layout_opts(&self, parent_opts: &LayoutOptions) -> LayoutOptions {
        Element::create_layout_opts(self.element(), parent_opts)
    }

    fn get_child(&self, child: usize) -> Option<&dyn NodeChild> {
        if let Some(child) = self.children().get(child) {
            Some(child)
        } else {
            None
        }
    }
}

impl NodeChild for String {
    fn paint(&self) -> Option<PaintDetails> {
        Some(PaintDetails {
            text: Some(self.clone()),
            ..Default::default()
        })
    }

    fn create_layout_opts(&self, parent_opts: &LayoutOptions) -> LayoutOptions {
        LayoutOptions {
            layout_ty: LayoutType::Text(self.clone()),
            text_size: parent_opts.text_size,
            ..Default::default()
        }
    }

    fn get_child(&self, _child: usize) -> Option<&dyn NodeChild> {
        None
    }
}
