use yew_router::prelude::*;
use yew::prelude::*;

use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};

use std::collections::HashMap;
use itertools::Itertools;
use std::sync::*;
// use reqwest::*;
use serde_json::json;
use wasm_bindgen::JsCast;
use serde::{Serialize,Deserialize};
use yew::platform::spawn_local;

use web_sys::HtmlInputElement;
mod error_modal;
mod home_page;
mod view_sheet_page;
mod find_sheet_page;
mod new_sheet_page;
mod shared_structs;
mod use_bool_toggle;
mod edit_members_list;
mod transaction;
mod fork_sheet_redirect;
use error_modal::*;
use edit_members_list::*;
use transaction::*;
use use_bool_toggle::*;
use shared_structs::*;
use home_page::*;
use find_sheet_page::*;
use new_sheet_page::*;
use view_sheet_page::*;
use fork_sheet_redirect::*;


async fn perfom_server_request<T: Serialize, K: serde::de::DeserializeOwned + std::fmt::Debug>(payload: T, request_type:&str) -> Result<K,String>{
    let client = reqwest::Client::new();
    let mut url = "http://127.0.0.1:7525/".to_owned();
    url.push_str(request_type);
    let response_unfinished = client.post(&url)
            .json(&payload)
            .send().await.map_err(|m| m.to_string())?;
    if response_unfinished.status() ==reqwest::StatusCode::OK {
        let response = response_unfinished.json::<K>().await.map_err(|k| k.to_string())?;
        Ok(response)
    }else {
        let response = response_unfinished.text().await.unwrap();
        Err(response)
    }
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/newsheet")]
    NewSheet,
    #[at("/findsheet")]
    FindSheet,
    #[at("/sheet/:uuid")]
    ViewSheet {uuid: String},
    #[at("/fork_sheet_redirect/:uuid")]
    ForkSheetRedirect {uuid: String},
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { 
            <HomePage />
        },
        Route::NewSheet => html! {
            <NewSheetPage />
        },
        Route::FindSheet => html! {
            <FindSheetPage />
        },
        Route::ViewSheet { uuid } => html! {
            <ViewSheetPage {uuid}/>
        },
        Route::ForkSheetRedirect { uuid } => html! {
            <ForkSheetRedirect {uuid}/>
        },
        Route::NotFound => html! { <h1>{ "404 Page not found" }</h1> },
    }
}

#[function_component(Main)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Main>::new().render();
}
