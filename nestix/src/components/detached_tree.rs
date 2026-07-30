use nestix_macros::{component, layout, props};

use crate::{Element, Layout, components::Fragment};

/// Props for [`DetachedTree`].
#[props(debug)]
#[derive(Debug)]
pub struct DetachedTreeProps {
    /// Logical descendants owned by the detached tree.
    #[props(default)]
    pub children: Layout,
}

/// Mounts a logical subtree without exposing its host handles to the surrounding tree.
///
/// Descendants can resolve parent and predecessor handles produced within this tree,
/// while placement outside the boundary treats the entire tree as having no host output.
#[component]
pub fn DetachedTree(props: &DetachedTreeProps, element: &Element) -> Element {
    element.detach_host_tree();

    layout! {
        Fragment(.children = props.children.clone())
    }
}
