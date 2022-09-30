

use crate::*;
// from https://stackoverflow.com/questions/53866508/how-to-make-a-public-struct-where-all-fields-are-public-without-repeating-pub
// 
// It really would be great if rust added a way to indicate that all elements of a struct are
// pub... 
//
macro_rules! pub_struct {
    ($name:ident {$($field:ident: $t:ty,)*}) => {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)] // ewww
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}
pub_struct!( GetSheetRequest {
    sheet_uuid : String,
});

pub_struct! (UpdateSheetRequest {
    sheet : Sheet,
});

pub_struct! (ForkSheetRequest {
    sheet_uuid : String,
});

pub_struct!( ForkSheetResponse {
    sheet_uuid : String,
});

pub_struct!( CreateSheetRequest {
    sheet_name : String,
});

pub_struct!( CreateSheetResponse {
    sheet_uuid : String,
});

pub_struct!( Person {
    uuid : String,
    name : String, 
    color : String,
});

pub_struct!( LineItem {
    //receiver_uuid : String,
    locked_payment_amount : Option<f64>,
});

pub_struct!( Transaction {
    transaction_uuid : String,
    name : String, 
    date_added : chrono::DateTime::<Utc>,
    giver_uuid : String,
    transaction_amount : f64,
    line_items : HashMap<String,LineItem>,
});

pub_struct!( Sheet{
    uuid : String,
    name : String,
    version_number : usize,
    members: HashMap<String,Person>,
    transactions : HashMap<String,Transaction>,
});
