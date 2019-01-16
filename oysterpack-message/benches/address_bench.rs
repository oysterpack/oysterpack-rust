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

#[macro_use]
extern crate criterion;

use criterion::Criterion;
use oysterpack_message::security::Address;

use std::{
    fs,
    io::{prelude::*, BufReader},
    path::PathBuf,
};

criterion_group!(benches, crypto_bench);

criterion_main!(benches);

fn data() -> Vec<u8> {
    let mut cargo_toml_path = PathBuf::new();
    cargo_toml_path.push(env!("CARGO_MANIFEST_DIR"));
    cargo_toml_path.push("Cargo.toml");
    let file = fs::File::open(cargo_toml_path.as_path()).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    println!("data.len() = {}", contents.len());
    Vec::from(contents)
}

/// using precomputed keys to encrypt / decrypt is ~10x faster
fn crypto_bench(c: &mut Criterion) {
    sodiumoxide::init().unwrap();
    let (_client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let (server_public_key, _server_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let bytes = data();
    let nonce = sodiumoxide::crypto::box_::gen_nonce();
    c.bench_function("seal", move |b| {
        b.iter(|| {
            sodiumoxide::crypto::box_::seal(
                &bytes,
                &nonce,
                &server_public_key,
                &client_private_key,
            );
        })
    });

    let bytes = data();
    let (_client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let (server_public_key, _server_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let server_addr = Address::from(server_public_key);
    let key = server_addr.precompute_key(&client_private_key);
    c.bench_function("seal_precomputed", move |b| {
        b.iter(|| {
            sodiumoxide::crypto::box_::seal_precomputed(&bytes, &nonce, &key);
        })
    });

    let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let server_addr = Address::from(server_public_key);
    let key = server_addr.precompute_key(&client_private_key);
    let encrypted_data = sodiumoxide::crypto::box_::seal_precomputed(&data(), &nonce, &key);
    c.bench_function("open", move |b| {
        b.iter(|| {
            sodiumoxide::crypto::box_::open(
                &encrypted_data,
                &nonce,
                &client_public_key,
                &server_private_key,
            )
            .unwrap();
        })
    });

    let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let server_addr = Address::from(server_public_key);
    let key = server_addr.precompute_key(&client_private_key);
    let encrypted_data = sodiumoxide::crypto::box_::seal_precomputed(&data(), &nonce, &key);
    let client_addr = Address::from(client_public_key);
    let key = client_addr.precompute_key(&server_private_key);
    c.bench_function("open_precomputed", move |b| {
        b.iter(|| {
            sodiumoxide::crypto::box_::open_precomputed(&encrypted_data, &nonce, &key).unwrap();
        })
    });
}
