/*
 * Copyright 2018 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use exitfailure::ExitFailure;
use structopt::StructOpt;

#[cfg_attr(tarpaulin, skip)]
fn main() -> Result<(), ExitFailure> {
    Command::from_args().execute()?;
    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "oysterpack-ulid",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
enum Command {
    #[structopt(name = "generate")]
    /// generate new ULID
    Generate,
    #[structopt(name = "parse")]
    /// parse ULID represented as either a string or u128 number
    /// - ULID strings are leniently parsed as specified in Crockford Base32 Encoding (https://crockford.com/wrmg/base32.html)
    Parse { ulid: String },
}

impl Command {
    fn execute(self) -> Result<(), failure::Error> {
        match self {
            Command::Generate => {
                print_ulid(oysterpack_uid::ULID::generate());
                Ok(())
            }
            Command::Parse { ulid } => match ulid.parse::<oysterpack_uid::ULID>() {
                Ok(ulid) => {
                    print_ulid(ulid);
                    Ok(())
                }
                Err(err) => match ulid.parse::<u128>() {
                    Ok(ulid_u128) => {
                        print_ulid(oysterpack_uid::ULID::from(ulid_u128));
                        Ok(())
                    }
                    Err(_) => Err(err.into()),
                },
            },
        }
    }
}

fn print_ulid(ulid: oysterpack_uid::ULID) {
    let ulid_u128: u128 = ulid.into();
    println!("{:?}", (ulid.to_string(), ulid_u128, ulid.datetime()));
}
