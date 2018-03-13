#[macro_use]
extern crate serde_derive;

extern crate byteorder;
extern crate serde;
extern crate serde_json;
extern crate toodle;

use std::io::{self, Read, StdinLock, StdoutLock, Write};

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use toodle::{Timespec, Toodle, Uuid};
use toodle::items::Item;

#[derive(Serialize, Debug)]
enum Error {
    IOError,
    BadJSON,
    BadRequest,
    UpdateItemFailed,
}


#[derive(Serialize, Deserialize, Debug)]
struct ItemInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    uuid: Option<String>,

    name: String,

    #[serde(rename = "completionDate", skip_serializing_if = "Option::is_none")]
    completion_date: Option<i64>,
}

impl From<Item> for ItemInfo {
    fn from(item: Item) -> Self {
        let completion_date = item.completion_date.as_ref().map_or(0, to_millis);
        ItemInfo {
            uuid: Some(item.uuid.hyphenated().to_string()),
            name: item.name.clone(),
            completion_date: Some(completion_date),
        }
    }
}

impl Into<Item> for ItemInfo {
    fn into(self) -> Item {
        let completion_date = self.completion_date.map(from_millis);
        Item {
            id: None,
            uuid: self.uuid
                .clone()
                .and_then(|uuid| Uuid::parse_str(&uuid).ok())
                .unwrap_or_else(|| Uuid::nil()),
            name: self.name.clone(),
            completion_date,
            labels: Vec::new(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum RequestBody {
    CreateTodo(ItemInfo),
    GetTodos,
    TodoChangeName { uuid: String, name: String },
    TodoChangeCompletionDate {
        uuid: String,

        #[serde(rename = "completionDate")]
        completion_date: i64,
    },
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
}

fn to_millis(time: &Timespec) -> i64 {
    time.sec * 1000 + (time.nsec / 1000000) as i64
}

fn from_millis(millis: i64) -> Timespec {
    Timespec::new(millis / 1000, (millis % 1000 * 1000000) as i32)
}

fn main() {
    let mut toodle = Toodle::new("./toodlext.sqlite".to_owned()).unwrap();

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
                            .update_item_by_uuid(&uuid, Some(name), None)
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
                                                 Some(from_millis(completion_date)))
                            .map(|item| ResponseBody::UpdateTodo(item.into()))
                            .map_err(|_err| Error::UpdateItemFailed)
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
