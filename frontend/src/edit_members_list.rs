use crate::*;

#[derive(PartialEq, Properties, Clone)]
pub struct EditPeopleProps {
    pub people : HashMap<String,Person>, 
    pub return_callback : Callback<()>,
    pub finished_callback : Callback::<(HashMap<String,Person>,Callback<Result<Sheet,String>>)>,
}

#[function_component(EditPeopleComponent)]
pub fn editpeoplecomponent(props : &EditPeopleProps) -> Html {
    let original_people = props.people.clone();
    let people = use_state(|| props.people.clone());

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

    let on_result = Callback::<Result<Sheet,String>>::from({
        let return_callback = props.return_callback.clone();
        let people = people.clone();
        move |res:Result<Sheet, String>| {
            if  res.is_ok(){
                return_callback.emit(());
            } else if let Err(msg) = res{
                log::info!("Error! {:?}",&msg );
                show_error.emit(msg);
                people.set(original_people.clone());

            }
        }
    });
    let onfinished = {
        let finished_callback = props.finished_callback.clone();
        
        let people = people.clone();
        move |_| {
            finished_callback.emit(((*people).clone(), on_result.clone()));
        }
    };
    let on_remove_person = Callback::<String>::from({
        let people = people.clone();
        move |uuid : String| {
            let mut people_clone = (*people).clone();
            people_clone.remove(&uuid);
            people.set(people_clone);
        }
    });
    let add_person_keypress = {
        let people = people.clone();
        move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let input:HtmlInputElement = e.target_unchecked_into();
                let name = input.value();
                input.set_value("");
                if !name.is_empty() {
                    log::info!("Creating {:?}", name);
                    let mut people_copy = (*people).clone();
                    let uuid = uuid::Uuid::new_v4().to_string();
                    people_copy.insert(uuid.clone(), 
                        Person { 
                            uuid, 
                            name, 
                            color: "#FF0000".to_owned() 
                        }
                    );
                    people.set(people_copy);
                    
                }
            }
        }
    };

    html! {
        <div>
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
        <div class="centered">
            <h1> {"Edit Sheet Members"}</h1>
            </div>
            <div class="new-person-div">
                <h2>{"Add a new Member:"}</h2>
                <input 
                    class="new-person-textbox"
                    placeholder="Ex: Mary, Bob, Xavier, etc."
                    onkeypress={add_person_keypress}
                />
                <p>{"(Press Enter after each name)"}</p>
            </div>
            <hr/>
            <div class="list-of-people">
                <h2>{"List of members:"}</h2>
                <ol>
                {
                    for (*people).clone().values()
                    .sorted_by_key(|v| v.name.clone().to_uppercase())
                    .map(|v| 
                        html!{
                            <li>
                            <PersonListItemComponent
                                person={(*v).clone()}
                                on_remove_person={on_remove_person.clone()}
                                />
                            </li>
                        }
                    )
                }
                </ol>
            </div>
            <button onclick={onfinished}> {"Save Changes and Return"} </button>
        </div>
    }
}


#[derive(PartialEq, Properties, Clone)]
pub struct PersonListItemProps {
    pub person : Person,
    pub on_remove_person : Callback<String>,
}

#[function_component(PersonListItemComponent)]
fn personlistitemcomponent(props : &PersonListItemProps) -> Html {
    let person = props.person.clone();
    let on_remove_self = Callback::from({
        let on_remove_person = props.on_remove_person.clone();
        move |_| {
            on_remove_person.emit(person.uuid.clone());
        }
    });
    html! {
        <div class="personcomp">
            <div class="personcomponentflex">
                <p> {person.name.clone()} </p>
                <button onclick={on_remove_self}> {"X"} </button>
            </div>
        </div>
    }
}
