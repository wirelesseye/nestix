use std::ops::Index;

use crate::Element;

#[derive(Debug, Clone)]
enum LayoutInner {
    Empty,
    Element(Element),
    ElementList(Vec<Element>),
}

#[derive(Debug, Clone)]
pub struct Layout(LayoutInner);

impl Layout {
    pub fn len(&self) -> usize {
        match &self.0 {
            LayoutInner::Empty => 0,
            LayoutInner::Element(_) => 1,
            LayoutInner::ElementList(elements) => elements.len(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Element> {
        match &self.0 {
            LayoutInner::Element(element) if index == 0 => Some(element),
            LayoutInner::ElementList(elements) => elements.get(index),
            _ => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Element> {
        self.into_iter()
    }

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

pub enum Iter<'a> {
    Empty,
    Element(std::iter::Once<&'a Element>),
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

pub enum IntoIter {
    Empty,
    Element(std::iter::Once<Element>),
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
