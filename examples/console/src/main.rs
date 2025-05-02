use bon::Builder;
use nestix::{component, components::fragment::Fragment, create_app_model, layout, Element, Props};

#[derive(Clone, PartialEq, Props, Builder, Debug)]
struct AppProps {
    data: i32,
}

#[component]
fn Application(props: &AppProps) -> Element {
    println!("render Application");

    layout! {
        Fragment {
            Window(.data = props.data),
            Window(.data = 37)
        }
    }
}

#[component]
fn Window(props: &AppProps) {
    println!("render Window {}", props.data);
}

fn main() {
    let app_model = create_app_model();
    let application = layout! {
        Application(.data = 42)
    };
    app_model.render(application);

    let application = layout! {
        Application(.data = 43)
    };
    app_model.render(application);
}
