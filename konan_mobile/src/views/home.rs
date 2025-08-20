use std::{
    collections::HashMap,
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use dioxus::prelude::*;

const HOME_CSS: Asset = asset!("/assets/styling/home.css");

#[component]
pub fn Home() -> Element {
    let mut form_state = use_signal(|| FormState::Pending);
    let mut content = use_signal(|| "".to_string());
    let mut printed_history = use_signal(|| HashMap::<u64, String>::new());

    rsx! {
        document::Link { rel: "stylesheet", href: HOME_CSS }

        {match form_state.read().clone() {
            FormState::Success => rsx! { h6 { "Printed" } },
            FormState::Pending => rsx! { },
            FormState::Failure(failure_state) => rsx! { h6 { "{failure_state:?}" } },
        }}

        div {
            class: "printed_history--container",
            ul {
                class: "printed_history--list",
                for (time_stamp, printed) in printed_history().iter() {
                    li {
                        key: "{time_stamp}",
                        button { class:"printed_history--item-btn", onclick: {
                            let mut content = content.clone();
                            let printed = printed.clone();
                            move |_| {
                                content.set(printed.clone());
                            }
                        }, "{printed}" }
                    }
                }
            }
        }

        form {
            class:"print-form",
            onsubmit: move |event| async move {
                event.prevent_default();
                let form_data = event.values();
                match form_data.get("content") {
                    Some(content_to_print) => {
                        match print_form_content(content_to_print.as_value()).await {
                            Ok(_) => {
                                printed_history.with_mut(|history| {
                                    history.insert(get_time_stamp(), content.cloned());
                                });
                                content.set("".to_string());
                                form_state.set(FormState::Success)
                            },
                            Err(e) => form_state.set(FormState::Failure(FailureState::PrintError(e.to_string()))),
                        }
                    },
                    None => form_state.set(FormState::Failure(FailureState::EmptyContent)),
                }
            },

            textarea {
                id: "content", name: "content",
                placeholder: "Type here...", autocomplete: "off", autofocus: true,
                value: "{content}",
                oninput: move |event| content.set(event.value())
            }

            button { id: "submit", r#type: "submit", disabled:content.read().len() == 0, "Print"  }
        }
    }
}

async fn print_form_content(content: String) -> Result<()> {
    let printer = rongta::establish_rongta_printer()?;
    rongta::print(
        rongta::Template {
            content: vec![content],
            ..rongta::Template::default()
        },
        printer,
    )
}

fn get_time_stamp() -> u64 {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_epoch.as_secs()
}

#[derive(Debug, Clone)]
enum FormState {
    Success,
    Pending,
    Failure(FailureState),
}

#[derive(Debug, Clone)]
enum FailureState {
    EmptyContent,
    PrintError(String),
}

impl Display for FailureState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
