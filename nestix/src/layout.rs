use std::ops::Index;

use crate::Element;

#[derive(Debug, Clone)]
enum LayoutInner {
    Empty,
    Element(Element),
    ElementList(Vec<Element>),
}

/// A component output containing zero, one, or many elements.
///
/// Layouts are used for children and component return values. They can be
/// created from `()`, a single [`Element`], an `Option<Element>`, or a vector of
/// elements.
#[derive(Debug, Clone)]
pub struct Layout(LayoutInner);

impl Layout {
    /// Returns the number of elements in this layout.
    pub fn len(&self) -> usize {
        match &self.0 {
            LayoutInner::Empty => 0,
            LayoutInner::Element(_) => 1,
            LayoutInner::ElementList(elements) => elements.len(),
        }
    }

    /// Returns the element at `index`, or `None` when it is out of bounds.
    pub fn get(&self, index: usize) -> Option<&Element> {
        match &self.0 {
            LayoutInner::Element(element) if index == 0 => Some(element),
            LayoutInner::ElementList(elements) => elements.get(index),
            _ => None,
        }
    }

    /// Iterates over the elements in this layout by reference.
    pub fn iter(&self) -> impl Iterator<Item = &Element> {
        self.into_iter()
    }

    /// Converts this layout into its contained elements.
    pub fn into_elements(self) -> Vec<Element> {
        match self.0 {
            LayoutInner::Empty => vec![],
            LayoutInner::Element(element) => vec![element],
            LayoutInner::ElementList(elements) => elements,
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self(LayoutInner::Empty)
    }
}

impl From<()> for Layout {
    fn from(_: ()) -> Self {
        Self(LayoutInner::Empty)
    }
}

impl From<Element> for Layout {
    fn from(value: Element) -> Self {
        Self(LayoutInner::Element(value))
    }
}

impl From<Option<Element>> for Layout {
    fn from(value: Option<Element>) -> Self {
        match value {
            Some(element) => Self(LayoutInner::Element(element)),
            None => Self(LayoutInner::Empty),
        }
    }
}

impl From<Option<Vec<Element>>> for Layout {
    fn from(value: Option<Vec<Element>>) -> Self {
        match value {
            Some(elements) => Self(LayoutInner::ElementList(elements)),
            None => Self(LayoutInner::Empty),
        }
    }
}

impl From<Vec<Element>> for Layout {
    fn from(value: Vec<Element>) -> Self {
        Self(LayoutInner::ElementList(value))
    }
}

/// Borrowing iterator over a [`Layout`].
pub enum Iter<'a> {
    /// Iterator for an empty layout.
    Empty,
    /// Iterator for a single-element layout.
    Element(std::iter::Once<&'a Element>),
    /// Iterator for a many-element layout.
    ElementList(std::slice::Iter<'a, Element>),
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Element;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Empty => None,
            Iter::Element(once) => once.next(),
            Iter::ElementList(iter) => iter.next(),
        }
    }
}

impl<'a> IntoIterator for &'a Layout {
    type Item = &'a Element;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match &self.0 {
            LayoutInner::Empty => Iter::Empty,
            LayoutInner::Element(element) => Iter::Element(std::iter::once(element)),
            LayoutInner::ElementList(elements) => Iter::ElementList(elements.into_iter()),
        }
    }
}

/// Owning iterator over a [`Layout`].
pub enum IntoIter {
    /// Iterator for an empty layout.
    Empty,
    /// Iterator for a single-element layout.
    Element(std::iter::Once<Element>),
    /// Iterator for a many-element layout.
    ElementList(<Vec<Element> as IntoIterator>::IntoIter),
}

impl Iterator for IntoIter {
    type Item = Element;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter::Empty => None,
            IntoIter::Element(once) => once.next(),
            IntoIter::ElementList(iter) => iter.next(),
        }
    }
}

impl IntoIterator for Layout {
    type Item = Element;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            LayoutInner::Empty => IntoIter::Empty,
            LayoutInner::Element(element) => IntoIter::Element(std::iter::once(element)),
            LayoutInner::ElementList(elements) => IntoIter::ElementList(elements.into_iter()),
        }
    }
}

impl Index<usize> for Layout {
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        match &self.0 {
            LayoutInner::Element(element) if index == 0 => element,
            LayoutInner::ElementList(elements) => &elements[index],
            _ => panic!(
                "Layout index out of bounds: the length is {} but the index is {}",
                self.len(),
                index
            ),
        }
    }
}
