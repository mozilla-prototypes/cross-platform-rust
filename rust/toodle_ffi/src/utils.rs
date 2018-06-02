// Copyright 2016 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.
use std::os::raw::{
    c_char,
};
use mentat_ffi::utils::strings::{
    c_char_to_string as mentat_c_char_to_string,
};

pub mod time {
    use time::Timespec;
    use libc::time_t;

    pub fn optional_timespec(timestamp: *const time_t) -> Option<Timespec> {
        match timestamp.is_null() {
            true => None,
            false => Some(Timespec::new(unsafe { *timestamp as i64 }, 0))
        }
    }
}

pub fn c_char_to_string(cchar: *const c_char) -> String {
    mentat_c_char_to_string(cchar).to_string()
}
