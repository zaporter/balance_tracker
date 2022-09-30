use crate::*;

#[derive(PartialEq, Properties, Clone)]
pub struct ErrorModalProps {
    pub msg: String,
    pub callback: Callback<()>,
}

#[function_component(ErrorModal)]
pub fn errormodal(props: &ErrorModalProps) ->Html{
    let onclick = {
        let callback = props.callback.clone();
        move |_| {
            callback.emit(());
        }
    };
    html!{
        <div class="modal">
            <div class="modal-content">
                <h1>{"Error"}</h1>
                <p>{props.msg.clone()}</p>
                <button {onclick}>{"Ok"}</button>
            </div>
        </div>
    }
}
