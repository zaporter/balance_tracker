use std::ops::Not;

use crate::*;
#[derive(PartialEq, Properties, Clone)]
pub struct TransactionProps {
    pub people : HashMap<String, Person>,
    pub transaction : Transaction,
    pub finished_updating_transaction_callback : Callback<(Transaction,Callback<Result<Sheet,String>>)>,
    pub delete_transaction_callback : Callback<String>,
    pub notify_transaction_update : Callback<()>,
}

#[function_component(TransactionComponent)]
pub fn edittransactioncomponent(props : &TransactionProps) ->Html{
    let people = props.people.clone();
    let transaction = use_state(|| props.transaction.clone());
    let finished_updating_transaction_callback = props.finished_updating_transaction_callback.clone();
    let is_editing_transaction = use_state(|| transaction.name == "NEW_MARKER");
    let delete_transaction_callback = props.delete_transaction_callback.clone();
    if transaction.name == "NEW_MARKER" {
        let mut transaction_clone = (*transaction).clone();
        transaction_clone.name = "".to_owned();
        transaction.set(transaction_clone);
        
    }
    let select_payee_callback = Callback::<Person>::from({
        let transaction = transaction.clone();
        move |person:Person| {
            log::info!("Selected {:?}",&person.name);
            let mut transaction_clone = (*transaction).clone();
            transaction_clone.giver_uuid = person.uuid;
            transaction.set(transaction_clone);
        }
    });
    let edit_payee_callback = Callback::from({
        let transaction = transaction.clone();
        move |_| {
            let mut transaction_clone = (*transaction).clone();
            transaction_clone.giver_uuid = "".to_owned();
            transaction.set(transaction_clone);
        }
        
    });
    let add_receiver_callback = Callback::<Person>::from({
        let transaction = transaction.clone();
        move |person:Person| {
            let mut transaction_clone = (*transaction).clone();
            transaction_clone.line_items.insert( person.uuid, 
                LineItem { locked_payment_amount: None }
                );
            transaction.set(transaction_clone);
        }

    });
    let unfixed_cost_per_person : f64 = calculate_unfixed_costs_per_person(transaction.transaction_amount, transaction.line_items.values().into_iter().collect());
    let edited_line_item_callback = Callback::<(String,LineItem)>::from({
        let transaction = transaction.clone();
        move |(uuid,li)| {
            let mut transaction_clone = (*transaction).clone();
            transaction_clone.line_items.insert(uuid, li);
            transaction.set(transaction_clone);
        }
    });
    let removed_line_item_callback = Callback::<String>::from({
        let transaction = transaction.clone();
        move |uuid| {
            let mut transaction_clone = (*transaction).clone();
            transaction_clone.line_items.remove(&uuid);
            transaction.set(transaction_clone);
        }
    });
    let edit_transaction_amount_callback = Callback::<f64>::from({
        let transaction = transaction.clone();
        move |amount| {
            let mut transaction_clone = (*transaction).clone();
            transaction_clone.transaction_amount=amount;
            transaction.set(transaction_clone);
        }
    });
    let edit_transaction_name_callback = {
        let transaction = transaction.clone();
        move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut transaction_clone = (*transaction).clone();
            transaction_clone.name = input.value();
            transaction.set(transaction_clone);
        }
    };
    let start_editing_transaction_callback = Callback::from({
        let is_editing_transaction = is_editing_transaction.clone();
        move |_| {
            is_editing_transaction.set(true);
        }
    });

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
    let deal_with_results_from_server = Callback::<Result<Sheet,String>>::from({
        let is_editing_transaction = is_editing_transaction.clone();
        let notify_transaction_update = props.notify_transaction_update.clone();
        move |result: Result<Sheet, String>| {
            log::info!("Result: {:?}",&result);
            if result.is_ok() {
                is_editing_transaction.set(false);
                notify_transaction_update.emit(());
            }else if let Err(msg) = result{
                log::info!("Error! {:?}",&msg );
                show_error.emit(msg);
            }
        }
    });
    let save_transaction_callback = Callback::from({
        let is_editing_transaction = is_editing_transaction.clone();
        let transaction = transaction.clone();
        move |_| {
            finished_updating_transaction_callback.emit(((*transaction).clone(), deal_with_results_from_server.clone()));
        }
    });
    let delete_self_callback = Callback::from({
        let transaction = transaction.clone();
        move |_| {
            delete_transaction_callback.emit(transaction.transaction_uuid.clone());
        }
    });
    let are_you_sure_shown = use_state(|| false);
    html!{
        <div class="transaction">

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
            {
            if *is_editing_transaction {
                html!{
                    <div class="flexjustify">
                    <h2>{"Editing Transaction"}</h2>
                    <div class="rightjustify">
                        <button onclick={save_transaction_callback.clone()}> {"Save Changes"}</button>
                        {
                            if *are_you_sure_shown{
                                html!{
                                    <div>
                                        <button class="redbutton" onclick={delete_self_callback}>{"Yes, delete"}</button>
                                        <button onclick={let are_you_sure_shown = are_you_sure_shown.clone(); move|_| are_you_sure_shown.set(false)}>{"No, cancel"}</button>
                                    </div>
                                }
                            }
                            else {html!{
                                <button class="redbutton" onclick={let are_you_sure_shown = are_you_sure_shown.clone(); move |_| are_you_sure_shown.set(true)}>{"Delete Transaction"}</button>

                            }}
                        }
                    </div>
                    </div>
                }
            }else {
                html!{
                    <div class="flexjustify">
                        <h2>{transaction.name.clone()}</h2>
                        <button onclick={start_editing_transaction_callback.clone()}>{"Edit"}</button>
                    </div>
                    }
                }
            }
            {
            if *is_editing_transaction {
                html!{
                    <div>
            <h3>{"Transaction Name: "}</h3>
                <input 
                    value={transaction.name.clone()}
                    oninput={edit_transaction_name_callback}
                    />
                        </div>
                }
            }else {
                html!{
                    }
                }
            }

            {
            if *is_editing_transaction {
                html!{
                    <div>
                        <div >
                            <p>{format!("Date: {}",transaction.date_added.to_rfc2822())}</p>
                        </div>
                        <h3>{"Total Transaction Price:"}</h3>
                        <SmartNumberInput 
                            default={transaction.transaction_amount}
                            edited_callback={edit_transaction_amount_callback}
                            />
                    </div>
                }
            }else {
                html!{
                    <div class="flexjustify">
                        <h2>{format!("${}",transaction.transaction_amount)}</h2>
                        <div class="rightalign">
                            <p>{format!("Date: {}",transaction.date_added.to_rfc2822())}</p>
                        </div>
                    </div>
                    }
                }
            }
            {
             if *is_editing_transaction{
                html! {
                <h3>{"Person who paid for the transaction: "}</h3>
                }
             }else{html!{}}
            }
            {
            if *is_editing_transaction {
                if transaction.giver_uuid.is_empty() {
                    html! {
                        <ClickablePersonList
                            people={people.clone()}
                            selected_callback={select_payee_callback.clone()}
                            />
                    }
                } else{
                    html! {
                        <div class="flexjustify">
                            <h3>{people.get(&transaction.giver_uuid).unwrap().name.clone()}</h3>
                            <button class="smallbutton" onclick={edit_payee_callback}>{"Edit"}</button>
                        </div>
                    }
                }
            }else {
                html!{
                        <h3>{format!("Paid by: {}",people.get(&transaction.giver_uuid).unwrap().name.clone())}</h3>
                    }
                }
            }
            
            {
            if *is_editing_transaction {
                html!{
                <div>
                    <h3>{"People involved in the transaction:"}</h3>
                    <ClickablePersonList
                        people={people.clone().into_iter().filter(|(uuid,_)| !transaction.line_items.contains_key(&uuid.clone())).collect::<HashMap<String,Person>>()}
                        selected_callback={add_receiver_callback.clone()}
                        />
                </div>
                }
            } else{
                html!{}
            }
            }
            <ul>
            {
                for transaction.line_items.clone().into_iter()
                    .sorted_by_key(|(receiver_uuid,li)| people.get(&receiver_uuid.clone()).unwrap().name.clone())
                    .map(|(receiver_uuid,li)| 
                    html! {
                        <div>
                            <LineItemComponent 
                                line_item={li.clone()}
                                person={(*people.get(&receiver_uuid).unwrap()).clone()}
                                is_editing_transaction={*is_editing_transaction}
                                {unfixed_cost_per_person}
                                edited_callback={edited_line_item_callback.clone()}
                                removed_callback={removed_line_item_callback.clone()}
                                />
                        </div>
                    }
                )
            }
            </ul>
        </div>
    }
}
pub fn calculate_unfixed_costs_per_person(total_cost: f64, line_items :Vec<&LineItem>) ->f64{
    let mut res = total_cost;
    let mut num_unfixed = 0_f64;
    for val in line_items {
        if let Some(fixed_cost) = val.locked_payment_amount {
            res -= fixed_cost;
        }else {
            num_unfixed += 1.0;
        }
    }
    res/num_unfixed
}

#[derive(PartialEq, Properties, Clone)]
pub struct LineItemProps {
    pub line_item : LineItem,
    pub person : Person,
    pub is_editing_transaction : bool,
    pub unfixed_cost_per_person : f64,
    pub edited_callback : Callback::<(String,LineItem)>,
    pub removed_callback : Callback::<String>,
}

#[function_component(LineItemComponent)]
fn lineitemcomponent(props: &LineItemProps) -> Html{
    let person = props.person.clone();
    let line_item = props.line_item.clone();
    let is_editing_transaction = props.is_editing_transaction;
    let edited_callback = props.edited_callback.clone();
    let unfixed_cost_per_person = props.unfixed_cost_per_person;
    
    let toggle_locked = Callback::from({
        let line_item = line_item.clone();
        let edited_callback = edited_callback.clone();
        let uuid = person.uuid.clone();
        // let edited_callback = edited_callback.clone();
        move |_| {
            let mut line_item_clone = line_item.clone();
            if line_item_clone.locked_payment_amount.is_some(){
                line_item_clone.locked_payment_amount = None;
            } else {
                line_item_clone.locked_payment_amount=Some(unfixed_cost_per_person);
            }
            edited_callback.emit((uuid.clone(),line_item_clone));
        }
    });
    let on_update_fixed_price = Callback::<f64>::from({
        let line_item = line_item.clone();
        // let edited_callback = edited_callback.clone();
        let uuid = person.uuid.clone();
        move |new_val| {
            log::info!("Update fixed price");
            let mut line_item_clone = line_item.clone();
            line_item_clone.locked_payment_amount=Some(new_val);
            edited_callback.emit((uuid.clone(),line_item_clone));
        }
    });
    let remove_self = Callback::from({
        let uuid= person.uuid.clone();
        let removed_callback = props.removed_callback.clone();
        move |_| {
            removed_callback.emit(uuid.clone());
        }
    });
    html!{
        <div class="lineitem">
            <p>{person.name.clone()}</p>
            <div class="flexjustify">
            {
                if is_editing_transaction {
                    if let Some(fixed_cost) = line_item.locked_payment_amount{
                        html! {
                            <div class="lineiteminside">
                                <SmartNumberInput
                                    default={fixed_cost}
                                    edited_callback={on_update_fixed_price}
                                    />
                                <button onclick={toggle_locked.clone()}>{"🔒"}</button>
                            </div>
                        }
                    }else{
                        html! {
                            <div class="lineiteminside">
                                <p>{f64::trunc(props.unfixed_cost_per_person*100.0)/100.0}</p>
                                <button onclick={toggle_locked.clone()}>{"🔓"}</button>
                            </div>
                        }
                    }

                } else {
                    if let Some(fixed_cost) = line_item.locked_payment_amount{
                        html! {
                            <div>
                                <p>{format!("$ {}  🔒",f64::trunc(fixed_cost*100.0)/100.0)}</p>
                            </div>
                        }
                    }else{
                        html! {
                            <div>
                                <p>{format!("$ {}  🔓",f64::trunc(props.unfixed_cost_per_person*100.0)/100.0)}</p>
                            </div>
                        }
                    }
                }
            }
            {
            if is_editing_transaction {
                html! {
                    <button class="redbutton" onclick={remove_self}> {"X"} </button>
                }
            } else {
                html! {}
            }
            }
            </div>
        </div>
    }
}
#[derive(PartialEq, Properties, Clone)]
pub struct SmartNumberInputProps {
    pub default : f64,
    pub edited_callback : Callback<f64>,
}

#[function_component(SmartNumberInput)]
pub fn smartnumberinput(props: &SmartNumberInputProps) -> Html{
    let error_in_input = use_state(||false);
    let current_text_val = use_state(|| format!("{}",props.default));
    let edited_callback = props.edited_callback.clone();

    let onattemptsave = Callback::<HtmlInputElement>::from({
        let error_in_input = error_in_input.clone();
        
        move |input : HtmlInputElement| {
            let new_val = &input.value().parse::<f64>();
            if let Ok(new_val) = new_val {
                error_in_input.set(false);
                input.blur().unwrap();
                edited_callback.emit(*new_val);
            } else {
                error_in_input.set(true);
            }

        }
    });
    let onkeypress = {
        let onattemptsave = onattemptsave.clone();
        let current_text_val = current_text_val.clone();
        move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let input: HtmlInputElement = e.target_unchecked_into();
                current_text_val.set(input.value());
                onattemptsave.emit(input);
            }
        }
    };
    let onblur = Callback::from({
        let current_text_val = current_text_val.clone();
        move |e: FocusEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_text_val.set(input.value());
            onattemptsave.emit(input);
        }
    });
    html!{
        <div>
            <input
                value={(*current_text_val).clone()}
                onkeypress={onkeypress}
                onblur={onblur}
                style={if *error_in_input {"background-color:#ff0000"} else {""}}
                />
        </div>
    }
}

#[derive(PartialEq, Properties, Clone)]
pub struct ClickablePersonListProps {
    pub people : HashMap<String, Person>,
    pub selected_callback : Callback::<Person>,
}
#[function_component(ClickablePersonList)]
pub fn clickablepersonlist (props : &ClickablePersonListProps) -> Html{
    let people = props.people.clone();
    let selected_callback = props.selected_callback.clone();
    html! {
        <div>
            <ul>
                {
                    for people.values().clone().into_iter()
                        .sorted_by_key(|person| person.name.to_uppercase())
                        .map(|person|
                        html! {
                            <ClickablePersonListItem
                                person={(*person).clone()}
                                selected_callback={selected_callback.clone()}
                                />
                        }
                    )
                }
            </ul>
        </div>
    }
    
}
#[derive(PartialEq, Properties, Clone)]
pub struct ClickablePersonListItemProps {
    pub person :  Person,
    pub selected_callback : Callback::<Person>,
}

#[function_component(ClickablePersonListItem)]
fn clickablepersonlistitem (props: &ClickablePersonListItemProps) -> Html {
    let person = props.person.clone();
    let onclick = Callback::from({
        let selected_callback = props.selected_callback.clone();
        let person = person.clone();
        move |_| {
            selected_callback.emit(person.clone());
        }
    });
    html! {
        <div>
            <button {onclick}>{person.name.clone()} </button>
        </div>
    }

}
