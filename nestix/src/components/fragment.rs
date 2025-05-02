use bon::Builder;
use nestix_macros::Props;

use crate::Element;

use super::Component;

#[derive(PartialEq, Debug, Props, Builder)]
pub struct FragmentProps {
    pub children: Option<Vec<Element>>,
}

pub struct Fragment;

impl Component for Fragment {
    type Props = FragmentProps;

    fn render(app_model: &crate::AppModel, element: crate::Element) {
        let props = element.props.downcast_ref::<FragmentProps>().unwrap();
        if let Some(children) = &props.children {
            for child in children {
                app_model.add_child(child.clone());
            }
        }
    }
}
