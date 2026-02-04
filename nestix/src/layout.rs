use std::ops::Index;

use crate::Element;

#[derive(Debug, Clone, Default)]
pub struct Layout(Option<Vec<Element>>);

impl Layout {
    pub fn iter(&self) -> impl Iterator<Item = &Element> {
        self.into_iter()
    }

    pub fn into_elements(self) -> Option<Vec<Element>> {
        self.0
    }
}

impl From<()> for Layout {
    fn from(_: ()) -> Self {
        Self(None)
    }
}

impl From<Element> for Layout {
    fn from(value: Element) -> Self {
        Self(Some(vec![value]))
    }
}

impl From<Option<Element>> for Layout {
    fn from(value: Option<Element>) -> Self {
        match value {
            Some(element) => Self(Some(vec![element])),
            None => Self(None),
        }
    }
}

impl From<Option<Vec<Element>>> for Layout {
    fn from(value: Option<Vec<Element>>) -> Self {
        Self(value)
    }
}

impl From<Vec<Element>> for Layout {
    fn from(value: Vec<Element>) -> Self {
        Self(Some(value))
    }
}

impl<'a> IntoIterator for &'a Layout {
    type Item = &'a Element;
    type IntoIter = std::slice::Iter<'a, Element>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.as_deref().unwrap_or(&[]).iter()
    }
}

impl IntoIterator for Layout {
    type Item = Element;
    type IntoIter = std::vec::IntoIter<Element>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.unwrap_or_default().into_iter()
    }
}

impl Index<usize> for Layout {
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0.as_ref().unwrap()[index]
    }
}

mod tests {
    use crate::{
        Element, Fragment, Layout, PropValue, Shared, component, computed, create_state, layout, props
    };

    #[props]
    struct ContainerProps {
        children: Shared<dyn Fn(String) -> PropValue<Layout>>,
    }

    #[component]
    fn Container(props: &ContainerProps) {}

    #[props]
    struct TextProps {
        value: String,
    }

    #[component]
    fn Text(props: &TextProps) {}

    #[component]
    fn App() -> Element {
        let count = create_state(0);
        let count_str = computed!([count] || count.get().to_string());

        layout! {
            Fragment {
                Container [count_str] |value: String| {
                    if true {
                        Fragment {
                            Text(.value = count_str.clone())
                            Text(.value = value.clone())
                        }
                    }
                }
                Text(.value = count_str)
            }
        }
    }
}
