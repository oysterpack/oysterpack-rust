// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub trait Publisher<T> {
    fn publish(msg: T);
}

pub struct Message<T> {
    header: Header,
    message: Option<T>,
}

pub struct Request<T> {
    header: RequestHeader,
    message: Option<T>,
}

pub struct Response<T> {
    header: ResponseHeader,
    message: Option<T>,
}

pub struct Header {}

pub struct RequestHeader {
    header: Header
}

pub struct ResponseHeader {
    header: Header
}

