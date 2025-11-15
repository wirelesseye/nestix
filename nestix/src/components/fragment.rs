use crate::{Component, Element, prop::PropValue};

pub struct FragmentProps {
    pub children: PropValue<Option<Vec<Element>>>,
}

pub struct Fragment;

impl Component for Fragment {
    type Props = FragmentProps;

    fn render(model: &std::rc::Rc<crate::Model>, element: &crate::Element) {
        let props = element.props().downcast_ref::<Self::Props>().unwrap();

        if let Some(children) = props.children.get() {
            for child in children {
                model.render(&child);
            }
        }
    }
}
