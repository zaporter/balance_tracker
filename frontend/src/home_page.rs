use std::future;

use web_sys::HtmlElement;

use crate::*;
use std::pin::*;
use std::rc::*;

#[function_component(HomePage)]
pub fn homepage() -> Html {


    
    let error_status : UseStateHandle<Option<(String, Callback<()>)>>= use_state(|| None);
    let show_error = Callback::<String>::from({
        let error_status = error_status.clone();
        move |msg| {
            error_status.set(Some((msg,Callback::<()>::from({
                let error_status = error_status.clone();
                move |_| error_status.set(None)
            }))));
        }
    });

    let onkeypress = {
        let navigator = use_navigator().unwrap();
        let view_sheet_switch = Callback::<String>::from(move |uuid| navigator.push(&Route::ViewSheet { uuid }));
        let show_error = show_error.clone();
        move |e: KeyboardEvent| {
            let view_sheet_switch = view_sheet_switch.clone();
            let show_error = show_error.clone();
            if e.key() == "Enter" {
                let input: HtmlInputElement = e.target_unchecked_into();
                spawn_local(async move {
                    match perfom_server_request::<CreateSheetRequest, CreateSheetResponse>(CreateSheetRequest{sheet_name:input.value()}, "createsheet").await {
                        Ok(val) => {
                            log::info!("Received {:?}",val);
                            view_sheet_switch.emit(val.sheet_uuid);
                        },
                        Err(err) => {
                            log::info!("Error! {:?}", &err);  
                            show_error.clone().emit(err);
                        },
                    }
                });
            }
        }
    };
    let oncreatepress = {
        let navigator = use_navigator().unwrap();
        let view_sheet_switch = Callback::<String>::from(move |uuid| navigator.push(&Route::ViewSheet { uuid }));
        move |_| {
            let view_sheet_switch = view_sheet_switch.clone();
            let show_error = show_error.clone();
            let document = web_sys::window().unwrap().document().unwrap();
            let input = document.get_element_by_id("homeinputbox").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
                spawn_local(async move {
                    match perfom_server_request::<CreateSheetRequest, CreateSheetResponse>(CreateSheetRequest{sheet_name:input.value()}, "createsheet").await {
                        Ok(val) => {
                            log::info!("Received {:?}",val);
                            view_sheet_switch.emit(val.sheet_uuid);
                        },
                        Err(err) => {
                            log::info!("Error! {:?}", &err);  
                            show_error.clone().emit(err);
                        },
                    }
                });
        }
    };
    html! {
        <div class="page_holder">
        {
            if let Some((msg, callback)) = (*error_status).clone() {
                html! {
                    <ErrorModal
                        {msg}
                        {callback}
                        />
                }
            } else {html! {}}
        }
            <div class="home_page">
                <h1>{ "Simple Trusting Group Balance Sheet" }</h1>
                <div class="centered">
                    <p>{"This is a simple tool to help manage outstanding debts withing a group of trusting members"}</p>
                </div>
                <div class="home_page_button_area">
                    <h2>{"Try it now! What do you want to name your sheet?"}</h2>
                    <input 
                        id="homeinputbox"
                        class="new-sheet-textbox"
                        placeholder="Ex: House Groceries, Roadtrip, etc.."
                        {onkeypress}
                    />
                    <button onclick={oncreatepress}>{"Create a new sheet"}</button>
                </div>
                <div class="hbox">
                    <div class="home_page_hbox_side">
                        <h2>{"How to use it"}</h2>
                        <ol>
                            <li><p>{"Create a new sheet"}</p></li>
                            <li><p>{"Add some members"}</p></li>
                            <li><p>{"Share the URL (keep this saved!)"}</p></li>
                            <li><p>{"Add your transactions"}</p></li>
                        </ol>
                    </div>
                    <div class="home_page_hbox_side">
                        <h2>{"Limitations"}</h2>
                        <ul>
                            <li><p>{"If you lose the url you can no longer access the sheet"}</p></li>
                            <li><p>{"You must trust everyone who as access to the sheet"}</p></li>
                            <li><p>{"You cannot remove a member who is listed in transactions"}</p></li>
                        </ul>
                    </div>
                </div >
                // <ContextProvider<BasicData> context={(*data_ctx).clone()}>
                //     <DataDisplay />
                // </ContextProvider<BasicData>>
            </div>
        </div>
    }
}


