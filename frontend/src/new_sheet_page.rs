
use crate::*;
use web_sys::HtmlInputElement;
use yew::events::KeyboardEvent;


#[function_component(NewSheetPage)]
pub fn newsheetpage() -> Html {
// text field to get name
// button to generate it all
    let error_msg = use_state(|| "".to_owned());

    let change_error_msg = Arc::new(Callback::<String>::from({
        let error_msg = error_msg.clone();
        move |val| {
            error_msg.set(val);
        }
    }));
    let onkeypress = {
        let navigator = use_navigator().unwrap();
        let view_sheet_switch = Callback::<String>::from(move |uuid| navigator.push(&Route::ViewSheet { uuid }));
        let change_error_msg_clone = change_error_msg.clone();
        move |e: KeyboardEvent| {
            let view_sheet_switch = view_sheet_switch.clone();
            if e.key() == "Enter" {
                let input: HtmlInputElement = e.target_unchecked_into();
                let change_error_msg_clone = change_error_msg.clone();
                spawn_local(async move {
                    match perfom_server_request::<CreateSheetRequest, CreateSheetResponse>(CreateSheetRequest{sheet_name:input.value()}, "createsheet").await {
                        Ok(val) => {
                            log::info!("Received {:?}",val);
                            change_error_msg_clone.emit("".to_owned());
                            view_sheet_switch.emit(val.sheet_uuid);
                            //navigator.clone().push(&Route::ViewSheet { uuid: val.sheet_uuid });
                        },
                        Err(err) => {
                            log::info!("Error! {:?}", &err);  
                            change_error_msg_clone.emit(err);
                        },
                    }
                });
            }
        }
    };
    
    html! {
        <div class="newsheetpage">
            <h1>{ "New Sheet page" }</h1>
            <h2>{"What do you want to name your sheet?"}</h2>
            <input 
                class="new-sheet-textbox"
                placeholder="Ex: House Groceries, Roadtrip, etc.."
                {onkeypress}
            />
            <h2>{"Press Enter when you have selected a name"}</h2>
            <p>{(*error_msg).clone()}</p> 
            
        </div>
    }
}
