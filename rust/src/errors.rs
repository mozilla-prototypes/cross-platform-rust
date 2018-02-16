// Copyright 2016 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use rusqlite;

use mentat::errors as mentat;
use mentat::edn::UuidParseError;
use mentat::NamespacedKeyword;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        MentatError(mentat::Error, mentat::ErrorKind);
    }

    foreign_links {
        Rusqlite(rusqlite::Error);
        InvalidUuid(UuidParseError);
    }

    errors {
        UnknownAttribute(keyword: NamespacedKeyword) {
            description("Keyword not found")
            display("Keyword {} not found", keyword)
        }
        ItemNotFound(uuid: String) {
            description("Item not found")
            display("Item {} not found", uuid)
        }
        UnexpectedResultType(message: String) {
            description("An unexpected Result type was encountered")
            display("{}", message)
        }
    }
}
