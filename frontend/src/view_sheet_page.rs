use crate::*;

#[derive(PartialEq, Properties, Clone)]
pub struct ViewSheetProps {
    pub uuid: String,
}

#[derive(PartialEq, Debug, Clone)]
pub enum PageState {
    Loading,
    View(Sheet),
    EditPeople(HashMap<String, Person>),
}

#[function_component(ViewSheetPage)]
pub fn viewsheetpage(props: &ViewSheetProps) -> Html {
    //let sheet = use_state(|| None);
    let pagestate = use_state(|| PageState::Loading);
    let error_msg = use_state(|| "".to_owned());
    let should_reload_sheet = use_bool_toggle(true);
    let uuid = props.uuid.clone();

    let show_sheet_callback = Callback::<Result<Sheet, String>>::from({
        let pagestate = pagestate.clone();
        move |val: Result<Sheet, String>| {
            match val {
                Ok(sheet) => {
                    log::info!("Loaded Sheet {:?}", &sheet.uuid);
                    pagestate.set(PageState::View(sheet));
                }
                Err(msg) => {
                    log::info!("Error loading sheet! {:?}", msg);
                }
            };
        }
    });

    let get_sheet_from_backend = Callback::<(String, Callback<Result<Sheet, String>>)>::from({
        move |(uuid, callback): (String, Callback<Result<Sheet, String>>)| {
            // let (uuid, calback) = val;
            spawn_local(async move {
                callback.emit(
                    perfom_server_request::<GetSheetRequest, Sheet>(
                        GetSheetRequest { sheet_uuid: uuid },
                        "getsheet",
                    )
                    .await,
                );
            });
        }
    });
    // Yeah... This is insane.
    // This is far beyond what any reasonable human should ever write.
    // This function generates a function that accepts another function. That other function spawns
    // a async thread to read from the backend and then evaluate the inner function with that
    // result (while also passing through arbitrary data)
    // (all three of the functions are generic upon the same T.)
    fn get_sheet_from_backend_passthrough<T: 'static>(
    ) -> Callback<(T, String, Callback<(T, Result<Sheet, String>)>)> {
        Callback::<(T, String, Callback<(T, Result<Sheet, String>)>)>::from({
            move |(passthrough, uuid, callback): (
                T,
                String,
                Callback<(T, Result<Sheet, String>)>,
            )| {
                spawn_local(async move {
                    callback.emit((
                        passthrough,
                        perfom_server_request::<GetSheetRequest, Sheet>(
                            GetSheetRequest { sheet_uuid: uuid },
                            "getsheet",
                        )
                        .await,
                    ));
                });
            }
        })
    }
    let upload_sheet_to_backend = Callback::<(Sheet, Callback<Result<Sheet, String>>)>::from({
        move |(sheet, callback): (Sheet, Callback<Result<Sheet, String>>)| {
            spawn_local(async move {
                callback.emit(
                    perfom_server_request::<UpdateSheetRequest, Sheet>(
                        UpdateSheetRequest { sheet },
                        "updatesheet",
                    )
                    .await,
                );
            });
        }
    });
    let start_editing_people = Callback::<HashMap<String, Person>>::from({
        let pagestate = pagestate.clone();
        move |val| {
            pagestate.set(PageState::EditPeople(val));
        }
    });
    let finished_updating_people_callback = Callback::<HashMap<String, Person>>::from({
        let upload_sheet_to_backend = upload_sheet_to_backend.clone();
        let show_sheet_callback = show_sheet_callback.clone();
        let pagestate = pagestate.clone();
        let uuid = uuid.clone();
        let get_sheet_then_upload_intermediary = Callback::<(
            HashMap<String, Person>,
            Result<Sheet, String>,
        )>::from({
            move |(people, val_from_server): (HashMap<String, Person>, Result<Sheet, String>)| {
                log::info!("Got {:?} from server", val_from_server);
                match val_from_server {
                    Ok(mut sheet) => {
                        log::info!("Retrieved last version in order to update it {:?}", &sheet);
                        log::info!("People: {:?}", &people);
                        let sheet_clone = sheet.clone();
                        sheet.members = people;
                        sheet.version_number += 1;
                        upload_sheet_to_backend.emit((
                            sheet,
                            Callback::<Result<Sheet, String>>::from({
                                let pagestate = pagestate.clone();
                                move |val: Result<Sheet, String>| {
                                    match val {
                                        Ok(sheet) => {
                                            log::info!("Loaded Sheet {:?}", &sheet.uuid);
                                            pagestate.set(PageState::View(sheet));
                                        }
                                        Err(msg) => {
                                            pagestate.set(PageState::View(sheet_clone.clone()));
                                            log::info!("Error loading sheet! {:?}", msg);
                                        }
                                    };
                                }
                            }),
                        ));
                    }
                    Err(msg) => {
                        log::info!(
                            "During update was unable to retrieve. Dont change page. Message: {:?}",
                            msg
                        );
                    }
                };
            }
        });
        move |people| {
            get_sheet_from_backend_passthrough().emit((
                people,
                uuid.clone(),
                get_sheet_then_upload_intermediary.clone(),
            ));
        }
    });
    let delete_transaction_callback = Callback::<String>::from({
        let upload_sheet_to_backend = upload_sheet_to_backend.clone();
        let show_sheet_callback = show_sheet_callback.clone();
        let uuid = uuid.clone();
        let get_sheet_then_upload_intermediary = Callback::<(String, Result<Sheet, String>)>::from(
            {
                move |(transaction_uuid, val_from_server): (String, Result<Sheet, String>)| {
                    log::info!("Got {:?} from server", val_from_server);
                    match val_from_server {
                        Ok(mut sheet) => {
                            log::info!("Retrieved last version in order to update it {:?}", &sheet);
                            sheet.transactions.remove(&transaction_uuid);
                            sheet.version_number += 1;
                            // log::info!("People: {:?}",&people);
                            // sheet.members = people;
                            upload_sheet_to_backend.emit((sheet, show_sheet_callback.clone()));
                        }
                        Err(msg) => {
                            log::info!("During update was unable to retrieve. Dont change page. Message: {:?}",msg);
                        }
                    };
                }
            },
        );
        move |transaction_uuid| {
            get_sheet_from_backend_passthrough().emit((
                transaction_uuid,
                uuid.clone(),
                get_sheet_then_upload_intermediary.clone(),
            ));
        }
    });

    let finished_updating_transaction_callback = Callback::<(
        Transaction,
        Callback<Result<Sheet, String>>,
    )>::from({
        let upload_sheet_to_backend = upload_sheet_to_backend.clone();
        let show_sheet_callback = show_sheet_callback.clone();
        let uuid = uuid.clone();
        let get_sheet_then_upload_intermediary = Callback::<(
            (Transaction, Callback<Result<Sheet, String>>),
            Result<Sheet, String>,
        )>::from({
            move |((updated_transaction, desired_callback), val_from_server): (
                (Transaction, Callback<Result<Sheet, String>>),
                Result<Sheet, String>,
            )| {
                log::info!("Got {:?} from server", val_from_server);
                match val_from_server {
                    Ok(mut sheet) => {
                        log::info!("Retrieved last version in order to update it {:?}", &sheet);
                        //log::info!("People: {:?}",&people);
                        sheet.transactions.insert(
                            updated_transaction.transaction_uuid.clone(),
                            updated_transaction,
                        );
                        sheet.version_number += 1;
                        upload_sheet_to_backend.emit((sheet, desired_callback.clone()));
                    }
                    Err(msg) => {
                        log::info!(
                            "During update was unable to retrieve. Dont change page. Message: {:?}",
                            msg
                        );
                    }
                };
            }
        });
        move |updated_transaction| {
            get_sheet_from_backend_passthrough().emit((
                updated_transaction,
                uuid.clone(),
                get_sheet_then_upload_intermediary.clone(),
            ));
        }
    });
    let finished_updating_people_callback = Callback::<(
        HashMap<String, Person>,
        Callback<Result<Sheet, String>>,
    )>::from({
        let upload_sheet_to_backend = upload_sheet_to_backend.clone();
        let show_sheet_callback = show_sheet_callback.clone();
        let uuid = uuid.clone();
        let get_sheet_then_upload_intermediary = Callback::<(
            (HashMap<String, Person>, Callback<Result<Sheet, String>>),
            Result<Sheet, String>,
        )>::from({
            move |((updated_transaction, desired_callback), val_from_server): (
                (HashMap<String, Person>, Callback<Result<Sheet, String>>),
                Result<Sheet, String>,
            )| {
                log::info!("Got {:?} from server", val_from_server);
                match val_from_server {
                    Ok(mut sheet) => {
                        log::info!("Retrieved last version in order to update it {:?}", &sheet);
                        sheet.members = updated_transaction;
                        sheet.version_number += 1;
                        upload_sheet_to_backend.emit((sheet, desired_callback.clone()));
                    }
                    Err(msg) => {
                        log::info!(
                            "During update was unable to retrieve. Dont change page. Message: {:?}",
                            msg
                        );
                    }
                };
            }
        });
        move |updated_transaction| {
            get_sheet_from_backend_passthrough().emit((
                updated_transaction,
                uuid.clone(),
                get_sheet_then_upload_intermediary.clone(),
            ));
        }
    });
    let reload_main_page = Callback::<()>::from({
        let show_sheet_callback = show_sheet_callback.clone();
        let get_sheet_from_backend = get_sheet_from_backend.clone();
        let uuid = props.uuid.clone();
        move |_: ()| {
            get_sheet_from_backend.emit((uuid.clone(), show_sheet_callback.clone()));
        }
    });
    if *pagestate == PageState::Loading {
        reload_main_page.clone().emit(());
    }
    html! {
        <div class="view_page">
            {
                match (*pagestate).clone() {
                    PageState::Loading => {
                        html! {
                            <h1>{"Loading..."}</h1>
                        }
                    },
                    PageState::View(sheet) => {
                        html! {
                            <SheetViewComponent
                                key={sheet.version_number}
                                sheet={sheet.clone()}
                                start_editing_people_callback={start_editing_people}
                                {finished_updating_transaction_callback}
                                {delete_transaction_callback}
                                / >
                        }

                    },
                    PageState::EditPeople(people) => {
                        html! {
                            <EditPeopleComponent
                                {people}
                                return_callback={reload_main_page.clone()}
                                finished_callback={finished_updating_people_callback}/>
                        }
                    }
                }
            }
        </div>
    }
}

#[derive(PartialEq, Properties, Clone)]
pub struct SheetViewProps {
    pub sheet: Sheet,
    // pub should_reload_callback : Callback<bool>,
    pub start_editing_people_callback: Callback<HashMap<String, Person>>,
    pub delete_transaction_callback: Callback<String>,
    pub finished_updating_transaction_callback:
        Callback<(Transaction, Callback<Result<Sheet, String>>)>,
}

#[function_component(SheetViewComponent)]
fn sheetviewcomponent(props: &SheetViewProps) -> Html {
    let sheet = use_state(|| props.sheet.clone());
    let finished_updating_transaction_callback =
        props.finished_updating_transaction_callback.clone();

    let selected_breakdown_person = use_state(|| None);

    let set_breakdown_person_callback = Callback::<Option<Person>>::from({
        let selected_breakdown_person = selected_breakdown_person.clone();
        move |opper| {
            selected_breakdown_person.set(opper);
        }
    });
    let notify_transaction_update = Callback::<()>::from({
        let selected_breakdown_person = selected_breakdown_person.clone();
        move |_: ()| {
            log::info!("trans update");
            selected_breakdown_person.set(None);
        }
    });
    let create_new_transaction_and_edit = Callback::from({
        let sheet = sheet.clone();
        move |_| {
            let tranction_template = Transaction {
                transaction_uuid: uuid::Uuid::new_v4().to_string(),
                name: "NEW_MARKER".to_owned(),
                date_added: Utc::now(),
                giver_uuid: "".to_owned(),
                transaction_amount: 0_f64,
                line_items: HashMap::new(),
            };
            let mut sheet_clone = (*sheet).clone();
            sheet_clone.transactions.insert(
                tranction_template.transaction_uuid.clone(),
                tranction_template,
            );
            sheet.set(sheet_clone);
        }
    });
    let start_editing_people = {
        let sheet = sheet.clone();
        let start_editing_people_callback = props.start_editing_people_callback.clone();
        move |_| {
            start_editing_people_callback.emit(sheet.members.clone());
        }
    };
    let members_string: Option<String> = sheet
        .members
        .values()
        .clone()
        .into_iter()
        .map(|k| k.name.clone())
        .sorted()
        .reduce(|accum, item| format!("{}, {}", accum, item));
    let on_fork = {
        let sheet = sheet.clone();
        let navigator = use_navigator().unwrap();
        let view_sheet_switch = Callback::<String>::from(move |uuid| navigator.push(&Route::ForkSheetRedirect { uuid }));
        move |_| {
            let view_sheet_switch = view_sheet_switch.clone();
            let sheet_uuid = sheet.uuid.clone();
            spawn_local(async move {
                match perfom_server_request::<ForkSheetRequest, ForkSheetResponse>(ForkSheetRequest{sheet_uuid:sheet_uuid.clone()}, "forksheet").await {
                    Ok(val) => {
                        log::info!("Received {:?}",val);
                        view_sheet_switch.emit(val.sheet_uuid);
                    },
                    Err(err) => {
                        log::info!("Error! {:?}", &err);  
                    },
                }
            });
        }
    };
    html! {
        <div>
            <div class="sheetname">
                <p class="fork"><button onclick={on_fork}> {"Fork Sheet"} </button></p>
                <h1 class="text"> {format!("Sheet: {}",sheet.name.clone())}</h1>
            </div>
            <h2>{"Members:"}</h2>
            {
                if let Some(members_string) = members_string.clone() {
                    html!{
                        <p>{format!("Sheet Members: {}",members_string)} </p>
                    }
                }else{
                    html! {
                        <p>{"No members yet! Click on Add/Remove Members to add some"} </p>
                    }
                }
            }
            <button onclick={start_editing_people}> {"Add/Remove Members"} </button>
            <hr/>
            {
                if sheet.members.values().len()>0 {
                    html! {
                        <div>
                        <BreakdownComponent
                            sheet={(*sheet).clone()}
                            person={(*selected_breakdown_person).clone()}
                            select_person_callback={set_breakdown_person_callback.clone()}
                            />

                        <hr/>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
            {
                if let Some(person) = (*selected_breakdown_person).clone(){
                    html! {
                        <h2>{format!("{}'s Transactions:",person.name)}</h2>
                    }
                } else {
                    html! {
                        <h2>{"All Transactions:"}</h2>
                    }
                }
            }
            {
                if let Some(members_string) = members_string.clone() {
                    html!{
                    <button onclick={create_new_transaction_and_edit}> {"Add New Transaction"} </button>
                    }
                }else{
                    html! {
                    }
                }
            }
            <ul>
                {
                for (*sheet).clone().transactions.values().into_iter()
                // sort in decending order by date
                .sorted_by(|a,b| Ord::cmp(&b.date_added,&a.date_added))
                .filter(|s| {
                    if let Some(per) = &*selected_breakdown_person {
                        (per.uuid == s.giver_uuid) || (s.line_items.keys().contains(&per.uuid))
                    } else {
                        true
                    }
                })
                .map(|transaction|
                    html! {
                        <TransactionComponent
                            key={(transaction.transaction_uuid).clone()}
                            people={sheet.members.clone()}
                            transaction={(*transaction).clone()}
                            finished_updating_transaction_callback={finished_updating_transaction_callback.clone()}
                            delete_transaction_callback={props.delete_transaction_callback.clone()}
                            notify_transaction_update={notify_transaction_update.clone()}
                            />
                    }
                )
                }
            </ul>

        </div>
    }
}

#[derive(Clone)]
struct Dues {
    to: HashMap<String, f64>,
    from: HashMap<String, f64>,
    sum_difference: f64,
    self_paid: f64,
}

fn calculate_dues(target: &Person, sheet: &Sheet) -> Dues {
    let mut to = HashMap::new();
    let mut from = HashMap::new();
    let mut self_paid = 0.;
    // prefill maps
    for member in sheet.members.values() {
        if member.uuid != target.uuid {
            from.insert(member.uuid.clone(), 0.);
            to.insert(member.uuid.clone(), 0.);
        }
    }
    for transaction in sheet.transactions.values() {
        // all the transactions where you paid
        let unfixed_cost = calculate_unfixed_costs_per_person(
            transaction.transaction_amount,
            transaction.line_items.values().into_iter().collect(),
        );
        if transaction.giver_uuid == target.uuid {
            for (ower, amount) in transaction.line_items.clone().into_iter() {
                if let Some(fixed) = amount.locked_payment_amount {
                    if ower == target.uuid {
                        self_paid += fixed;
                        continue;
                    }
                    from.insert(ower.clone(), from.get(&ower).unwrap() + fixed);
                } else {
                    if ower == target.uuid {
                        self_paid += unfixed_cost;
                        continue;
                    }
                    from.insert(ower.clone(), from.get(&ower).unwrap() + unfixed_cost);
                }
            }
        }
        // all the transactions where you did not pay
        // if you are the target, ignore the transaction
        if transaction.giver_uuid == target.uuid {
            continue;
        }
        for (ower, amount) in transaction.line_items.clone().into_iter() {
            if ower == target.uuid {
                if let Some(fixed) = amount.locked_payment_amount {
                    to.insert(
                        transaction.giver_uuid.clone(),
                        to.get(&transaction.giver_uuid).unwrap() + fixed,
                    );
                } else {
                    to.insert(
                        transaction.giver_uuid.clone(),
                        to.get(&transaction.giver_uuid).unwrap() + unfixed_cost,
                    );
                }
            }
        }
    }
    let mut sum_difference = 0.;
    for member in sheet.members.values() {
        if member.uuid != target.uuid {
            let delta = from.get(&member.uuid).unwrap()-to.get(&member.uuid).unwrap();
            if delta > 0. {
                from.insert(member.uuid.clone(), delta);
                to.insert(member.uuid.clone(), 0.);
            }else {
                from.insert(member.uuid.clone(), 0.);
                to.insert(member.uuid.clone(), -1.*delta);
            }
            sum_difference-=delta;
        }
    }

    Dues {
        to,
        from,
        self_paid,
        sum_difference,
    }
}

#[derive(PartialEq, Properties, Clone)]
pub struct BreakdownProps {
    pub sheet: Sheet,
    pub person: Option<Person>,
    pub select_person_callback: Callback<Option<Person>>,
}

#[function_component(BreakdownComponent)]
fn breakdowncomponent(props: &BreakdownProps) -> Html {
    let sheet = props.sheet.clone();
    let person = props.person.clone();
    let select_person_callback = props.select_person_callback.clone();

    let selected_callback = Callback::<Person>::from({
        let select_person_callback = select_person_callback.clone();
        move |per| {
            select_person_callback.emit(Some(per));
        }
    });
    let unselected_callback = Callback::from({
        let select_person_callback = select_person_callback.clone();
        move |_| {
            select_person_callback.emit(None);
        }
    });
    let mut dues = None;
    if let Some(person) = &person {
        dues = Some(calculate_dues(person, &sheet));
    }

    html! {
        <div class="breakdown">
            <h2>{"View Detailed Breakdown:"}</h2>
            <ClickablePersonList
                people={sheet.members.clone()}
                selected_callback={selected_callback.clone()}
                />
            {
                if let Some(person) = person.clone(){
                    html! {
                        <div>
                            <p>{format!("For: {}",person.name)}</p>
                            <h3>{"You owe:"}</h3>
                            <ul>
                            {
                            for dues.clone().unwrap().to.into_iter().map(|(to_uuid,amount)|
                                html! {
                                    <p>{format!("${} to {}",f64::trunc(amount*100.0)/100.0,sheet.members.get(&to_uuid).unwrap().name.clone())}</p>
                                })
                            }
                            </ul>
                            <h3>{"You are owed:"}</h3>
                            <ul>
                            {
                            for dues.clone().unwrap().from.into_iter().map(|(from_uuid,amount)|
                                html! {
                                    <p>{format!("${} from {}",f64::trunc(amount*100.0)/100.0,sheet.members.get(&from_uuid).unwrap().name.clone())}</p>
                                })
                            }
                            </ul>
                            <p>{format!("You have paid ${} to people outside of this sheet",f64::trunc(dues.clone().unwrap().self_paid*100.0)/100.0)}</p>
                            <p>{format!("Your total balance is ${}",f64::trunc(dues.clone().unwrap().sum_difference*100.0)/100.0)}</p>
                            <button class="redbutton" onclick={unselected_callback}>{"Clear Breakdown Selection"}</button>
                        </div>
                    }
                }else {
                    html! {
                        <div>
                            <p>{"Nobody selected"}</p>
                        </div>
                    }
                }
            }

        </div>


    }
}
