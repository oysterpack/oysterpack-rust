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
extern crate rand;
extern crate uuid;

#[cfg(test)]
mod tests {
    #[test]
    fn test_uuid() {
        use uuid;
        use uuid::Uuid;

        let my_uuid = Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap();
        println!("Parsed a version {} UUID.", my_uuid.get_version_num());
        println!("{}", my_uuid);

        let id = Uuid::new_v4();
        println!("v4 {}", id);

        println!("v5 DNS {}", Uuid::new_v5(&uuid::NAMESPACE_DNS, &id.to_string()));
        println!("v5 OID {}", Uuid::new_v5(&uuid::NAMESPACE_OID, &id.to_string()));
        println!("v5 URL {}", Uuid::new_v5(&uuid::NAMESPACE_URL, &id.to_string()));
        println!("v5 X500 {}", Uuid::new_v5(&uuid::NAMESPACE_X500, &id.to_string()));

        assert_eq!(Uuid::new_v5(&uuid::NAMESPACE_DNS, &id.to_string()), Uuid::new_v5(&uuid::NAMESPACE_DNS, &id.to_string()));
        assert_eq!(Uuid::new_v5(&uuid::NAMESPACE_OID, &id.to_string()), Uuid::new_v5(&uuid::NAMESPACE_OID, &id.to_string()));
        assert_eq!(Uuid::new_v5(&uuid::NAMESPACE_URL, &id.to_string()), Uuid::new_v5(&uuid::NAMESPACE_URL, &id.to_string()));
        assert_eq!(Uuid::new_v5(&uuid::NAMESPACE_X500, &id.to_string()), Uuid::new_v5(&uuid::NAMESPACE_X500, &id.to_string()));
    }

    #[test]
    fn test_rand() {
        use rand;
        use rand::Rng;

        let mut rng = rand::thread_rng();
        if rng.gen() { // random bool
            println!("i32: {}, u32: {}", rng.gen::<i32>(), rng.gen::<u32>())
        }
        println!("{:?}", rng.gen::<(f64, bool)>());
    }
}
