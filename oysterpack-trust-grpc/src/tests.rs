/*
 * Copyright 2019 OysterPack Inc.
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

#[test]
fn network_interfaces() {
    for interface in pnet::datalink::interfaces() {
        println!("{:#?}", interface);
    }
}

#[test]
fn cert_pem_serialize_new_pem() {
    let subject_alt_names: &[_] = &["127.0.0.1".to_string()];

    let cert = rcgen::generate_simple_self_signed(subject_alt_names);
    let cert_pem = cert.serialize_pem();
    let cert_private_key_pem = cert.serialize_private_key_pem();

    println!("cert_pem\n{}", cert_pem);
    println!("cert_private_key_pem\n{}", cert_private_key_pem);

    let cert_pem_2 = cert.serialize_pem();
    let cert_private_key_pem_2 = cert.serialize_private_key_pem();

    println!("cert_pem_2\n{}", cert_pem_2);
    println!("cert_private_key_pem_2\n{}", cert_private_key_pem_2);

    assert_ne!(cert_pem, cert_pem_2);
    assert_eq!(cert_private_key_pem, cert_private_key_pem_2);
}
