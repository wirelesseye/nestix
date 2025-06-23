use std::rc::Rc;

use bon::Builder;
use nestix_macros::Props;

use crate::{AppModel, Element};

use super::Component;

#[derive(PartialEq, Debug, Props, Builder)]
pub struct FragmentProps {
    pub children: Option<Vec<Element>>,
}

pub struct Fragment;

impl Component for Fragment {
    type Props = FragmentProps;
    type Handle = ();

    fn render(app_model: &Rc<AppModel>, element: crate::Element) {
        let props = element.props.downcast_ref::<FragmentProps>().unwrap();
        if let Some(children) = &props.children {
            for child in children {
                app_model.push_child(child.clone());
            }
        }
    }
}
