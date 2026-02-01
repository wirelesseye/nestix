use std::ops::Index;

use crate::Element;

#[derive(Debug, Clone, Default)]
pub struct Children(Option<Vec<Element>>);

impl Children {
    pub fn iter(&self) -> impl Iterator<Item = &Element> {
        self.into_iter()
    }

    pub fn into_elements(self) -> Option<Vec<Element>> {
        self.0
    }
}

impl From<Element> for Children {
    fn from(value: Element) -> Self {
        Self(Some(vec![value]))
    }
}

impl From<Option<Vec<Element>>> for Children {
    fn from(value: Option<Vec<Element>>) -> Self {
        Self(value)
    }
}

impl From<Vec<Element>> for Children {
    fn from(value: Vec<Element>) -> Self {
        Self(Some(value))
    }
}

impl<'a> IntoIterator for &'a Children {
    type Item = &'a Element;
    type IntoIter = std::slice::Iter<'a, Element>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.as_deref().unwrap_or(&[]).iter()
    }
}

impl IntoIterator for Children {
    type Item = Element;
    type IntoIter = std::vec::IntoIter<Element>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.unwrap_or_default().into_iter()
    }
}

impl Index<usize> for Children {
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0.as_ref().unwrap()[index]
    }
}

mod tests {
    use nestix_macros::{callback, component, layout, props};
    use nestix_signal::{Computed, Shared, computed, create_state};

    use crate::{Children, Element, Fragment};

    #[props]
    struct ContainerProps {
        children: Shared<dyn Fn(String) -> Computed<Children>>,
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
            Container(
                .children = callback!(
                    move |value: String| computed!([count_str] || layout! {
                        Fragment {
                            Text(.value = count_str.clone())
                            Text(.value = value.clone())
                        }
                    }.into())
                ),
            )
        }
    }
}
