use crate::*;

#[derive(PartialEq, Properties, Clone)]
pub struct ForkSheetRedirectProps {
    pub uuid: String,
}

#[function_component(ForkSheetRedirect)]
pub fn forksheetredirectpage(props: &ForkSheetRedirectProps) -> Html {
    let uuid = props.uuid.clone();
    let navigator = use_navigator().unwrap();
    let view_sheet_switch = Callback::<String>::from(move |uuid| navigator.push(&Route::ViewSheet { uuid }));
    view_sheet_switch.emit(uuid);
    html!{
        <p>{"Forking.."}</p>
    }
}
