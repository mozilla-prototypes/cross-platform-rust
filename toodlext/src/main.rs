#[macro_use]
extern crate serde_derive;

extern crate byteorder;
extern crate serde;
extern crate serde_json;
extern crate toodle;

use std::collections::HashSet;
use std::io::{self, Read, StdinLock, StdoutLock, Write};
use std::iter::FromIterator;

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use toodle::{Store, Timespec, Toodle, Uuid};
use toodle::items::Item;
use toodle::labels::Label;

#[derive(Serialize, Debug)]
enum Error {
    IOError,
    BadJSON,
    BadRequest,
    LabelNotFound,
    ItemNotFound,
    UpdateItemFailed,
    UpdateLabelsFailed,
    NotImplemented,
}

#[derive(Serialize, Deserialize, Debug)]
struct LabelInfo {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
}

impl From<Label> for LabelInfo {
    fn from(label: Label) -> Self {
        LabelInfo {
            name: label.name.clone(),
            color: Some(label.color.clone()),
        }
    }
}

impl Into<Label> for LabelInfo {
    fn into(self) -> Label {
        Label {
            id: None,
            name: self.name.clone(),
            color: self.color.clone().unwrap_or_default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ItemInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    uuid: Option<String>,

    name: String,

    #[serde(rename = "dueDate", skip_serializing_if = "Option::is_none")]
    due_date: Option<i64>,

    #[serde(rename = "completionDate", skip_serializing_if = "Option::is_none")]
    completion_date: Option<i64>,

    labels: Option<Vec<LabelInfo>>,
}

impl From<Item> for ItemInfo {
    fn from(item: Item) -> Self {
        let due_date = item.due_date.as_ref().map_or(0, to_millis);
        let completion_date = item.completion_date.as_ref().map_or(0, to_millis);
        let label_infos = Some(item.labels
                                   .clone()
                                   .into_iter()
                                   .map(|label| label.into())
                                   .collect());
        ItemInfo {
            uuid: Some(item.uuid.hyphenated().to_string()),
            name: item.name.clone(),
            due_date: Some(due_date),
            completion_date: Some(completion_date),
            labels: label_infos,
        }
    }
}

impl Into<Item> for ItemInfo {
    fn into(self) -> Item {
        let due_date = self.due_date.map(from_millis);
        let completion_date = self.completion_date.map(from_millis);
        let labels = match self.labels {
            Some(labels) => labels.into_iter().map(|label| label.into()).collect(),
            None => Vec::new(),
        };
        Item {
            id: None,
            uuid: self.uuid
                .clone()
                .and_then(|uuid| Uuid::parse_str(&uuid).ok())
                .unwrap_or_else(|| Uuid::nil()),
            name: self.name.clone(),
            due_date,
            completion_date,
            labels,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum RequestBody {
    CreateTodo(ItemInfo),
    GetTodos,
    TodoChangeName { uuid: String, name: String },
    TodoChangeDueDate {
        uuid: String,

        #[serde(rename = "dueDate")]
        due_date: i64,
    },
    TodoChangeCompletionDate {
        uuid: String,

        #[serde(rename = "completionDate")]
        completion_date: i64,
    },
    TodoAddLabel { uuid: String, name: String },
    TodoRemoveLabel { uuid: String, name: String },
    RemoveTodo { uuid: String },
    AddLabel(LabelInfo),
    RemoveLabel { name: String },
    GetLabels,
}

#[derive(Deserialize, Debug)]
struct Request {
    id: i64,
    body: RequestBody,
}

impl Request {
    fn read_from(input: &mut StdinLock) -> Result<Request, Error> {
        let length = input
            .read_u32::<NativeEndian>()
            .map_err(|_err| Error::IOError)?;
        let mut message = input.take(length as u64);
        let mut buffer = Vec::with_capacity(length as usize);

        eprintln!("Reading request from browser");
        message
            .read_to_end(&mut buffer)
            .map_err(|_err| Error::IOError)?;

        serde_json::from_slice(&buffer).map_err(|err| {
            eprintln!("Error parsing request payload {:?}: {:?}",
                      String::from_utf8_lossy(&buffer),
                      err);
            Error::BadJSON
        })
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
enum Response {
    Ok { id: i64, body: ResponseBody },
    Err { id: i64, body: Error },
}

impl Response {
    fn write_to(&self, output: &mut StdoutLock) -> io::Result<()> {
        let message = serde_json::to_vec(self)?;
        output.write_u32::<NativeEndian>(message.len() as u32)?;
        output.write_all(&message)?;
        output.flush()
    }
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum ResponseBody {
    CreateTodo(ItemInfo),
    UpdateTodo(ItemInfo),
    GetTodos(Vec<ItemInfo>),
    GetLabels(Vec<LabelInfo>),
    RemoveTodo { uuid: String },
    AddLabel(LabelInfo),
    RemoveLabel { name: String },
}

fn update_item_labels(toodle: &mut Store,
                      uuid: &str,
                      to_add: Vec<String>,
                      to_remove: Vec<String>)
                      -> Result<ResponseBody, Error> {
    let uuid = Uuid::parse_str(&uuid).map_err(|_err| Error::ItemNotFound)?;
    let item = toodle
        .fetch_item(&uuid)
        .ok()
        .unwrap_or_default()
        .ok_or(Error::ItemNotFound)?;
    let existing_labels = toodle.fetch_labels().unwrap_or(vec![]);
    if to_add
           .iter()
           .any(|name| {
                    existing_labels
                        .iter()
                        .find(|label| &label.name == name)
                        .is_none()
                }) {
        return Err(Error::LabelNotFound);
    }
    let existing_item_label_names =
        HashSet::<&String>::from_iter(item.labels.iter().map(|label| &label.name));
    let mut new_labels = item.labels.clone();
    let mut labels_to_add = to_add
        .into_iter()
        .filter_map(|name| if existing_item_label_names.contains(&name) {
                        None
                    } else {
                        existing_labels
                            .iter()
                            .find(|label| label.name == name)
                            .cloned()
                    })
        .collect::<Vec<Label>>();
    new_labels.append(&mut labels_to_add);

    let item_label_names_to_remove = HashSet::<&String>::from_iter(to_remove.iter());
    new_labels.retain(|label| !item_label_names_to_remove.contains(&label.name));

    toodle
        .update_item(&item, None, None, None, Some(&new_labels))
        .and_then(|_| toodle.fetch_item(&uuid))
        .unwrap_or_default()
        .map(|item| ResponseBody::UpdateTodo(item.into()))
        .ok_or(Error::UpdateLabelsFailed)
}

fn to_millis(time: &Timespec) -> i64 {
    time.sec * 1000 + (time.nsec / 1000000) as i64
}

fn from_millis(millis: i64) -> Timespec {
    Timespec::new(millis / 1000, (millis % 1000 * 1000000) as i32)
}

fn main() {
    let mut toodle = Store::open("./toodlext.sqlite").unwrap();

    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut input = stdin.lock();
    let mut output = stdout.lock();

    loop {
        let response = Request::read_from(&mut input)
            .map(|request| {
                let result = match request.body {
                    RequestBody::CreateTodo(info) => {
                        toodle
                            .create_and_fetch_item(&info.into())
                            .unwrap_or_default()
                            .map(|item| ResponseBody::CreateTodo(item.into()))
                            .ok_or(Error::BadRequest)
                    }
                    RequestBody::GetTodos => {
                        toodle
                            .fetch_items()
                            .map(|items| {
                                     let infos =
                                         items.vec.into_iter().map(|item| item.into()).collect();
                                     ResponseBody::GetTodos(infos)
                                 })
                            .map_err(|_err| Error::BadRequest)
                    }
                    RequestBody::TodoChangeName { uuid, name } => {
                        toodle
                            .update_item_by_uuid(&uuid, Some(name), None, None)
                            .map(|item| ResponseBody::UpdateTodo(item.into()))
                            .map_err(|_err| Error::UpdateItemFailed)
                    }
                    RequestBody::TodoChangeDueDate { uuid, due_date } => {
                        toodle
                            .update_item_by_uuid(&uuid, None, Some(from_millis(due_date)), None)
                            .map(|item| ResponseBody::UpdateTodo(item.into()))
                            .map_err(|_err| Error::UpdateItemFailed)
                    }
                    RequestBody::TodoChangeCompletionDate {
                        uuid,
                        completion_date,
                    } => {
                        toodle
                            .update_item_by_uuid(&uuid,
                                                 None,
                                                 None,
                                                 Some(from_millis(completion_date)))
                            .map(|item| ResponseBody::UpdateTodo(item.into()))
                            .map_err(|_err| Error::UpdateItemFailed)
                    }
                    RequestBody::TodoAddLabel { uuid, name } => {
                        update_item_labels(&mut toodle, &uuid, vec![name], vec![])
                    }
                    RequestBody::TodoRemoveLabel { uuid, name } => {
                        update_item_labels(&mut toodle, &uuid, vec![], vec![name])
                    }
                    RequestBody::RemoveTodo { uuid } => Err(Error::NotImplemented),
                    RequestBody::AddLabel(info) => {
                        toodle
                            .create_label(info.name, info.color.unwrap_or_default())
                            .unwrap_or_default()
                            .map(|label| ResponseBody::AddLabel(label.into()))
                            .ok_or(Error::BadRequest)
                    }
                    RequestBody::RemoveLabel { name } => Err(Error::NotImplemented),
                    RequestBody::GetLabels => {
                        toodle
                            .fetch_labels()
                            .map(|labels| {
                                     let infos =
                                         labels.into_iter().map(|label| label.into()).collect();
                                     ResponseBody::GetLabels(infos)
                                 })
                            .map_err(|_err| Error::BadRequest)
                    }
                };
                match result {
                    Ok(body) => {
                        Response::Ok {
                            id: request.id,
                            body,
                        }
                    }
                    Err(err) => {
                        Response::Err {
                            id: request.id,
                            body: err,
                        }
                    }
                }
            })
            .ok();
        if let Some(r) = response {
            r.write_to(&mut output)
                .unwrap_or_else(|err| {
                                    eprintln!("Error handling request: {:?}", err);
                                });
        }
    }
}
