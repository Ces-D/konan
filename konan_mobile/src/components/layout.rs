use crate::Route;
use dioxus::prelude::*;

const LAYOUT_CSS: Asset = asset!("/assets/styling/layout.css");

#[component]
pub fn Layout() -> Element {
    rsx!(
       document::Link { rel: "stylesheet", href: LAYOUT_CSS }

       nav { id: "layout-nav", h1 { "Konan" } }
       main { id: "layout-main", Outlet::<Route> {} }
    )
}
