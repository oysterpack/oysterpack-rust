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

//! ## benchmark summary
//! Using precomputed keys, public-key encryption is just as fast as secret-key encryption.
//! Thus, there is no performance advantage to using secret-key authenticated encryption over
//! public-key authenticated encryption when using precomputed keys for public-key authenticated
//! encryption.

#![allow(warnings)]

#[macro_use]
extern crate criterion;

use criterion::Criterion;
use sodiumoxide::crypto::{box_, secretbox};

use std::{
    fs,
    io::{prelude::*, BufReader},
    path::PathBuf,
};

criterion_group!(
    benches,
    public_key_encryption_bench,
    public_key_decryption_bench,
    secret_key_encryption_bench,
    secret_key_decryption_bench
);

criterion_main!(benches);

fn data() -> Vec<u8> {
    let mut cargo_toml_path = PathBuf::new();
    cargo_toml_path.push(env!("CARGO_MANIFEST_DIR"));
    cargo_toml_path.push("Cargo.toml");
    let file = fs::File::open(cargo_toml_path.as_path()).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let mut data = Vec::from(contents.clone());
    for _ in 0..10 {
        let mut temp = Vec::from(contents.clone());
        data.append(&mut temp);
    }
    println!("data.len() = {}", data.len());
    data
}

fn public_key_encryption_bench(c: &mut Criterion) {
    sodiumoxide::init().unwrap();

    let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let nonce = box_::gen_nonce();
    let encryption_key = box_::precompute(&server_public_key, &client_private_key);
    let decryption_key = box_::precompute(&client_public_key, &server_private_key);
    let data = data();
    c.bench_function("box_::seal_precomputed", move |b| {
        b.iter(|| {
            let _ = sodiumoxide::crypto::box_::seal_precomputed(&data, &nonce, &encryption_key);
        })
    });
}

fn public_key_decryption_bench(c: &mut Criterion) {
    sodiumoxide::init().unwrap();

    let (client_public_key, client_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let (server_public_key, server_private_key) = sodiumoxide::crypto::box_::gen_keypair();
    let nonce = box_::gen_nonce();
    let encryption_key = box_::precompute(&server_public_key, &client_private_key);
    let decryption_key = box_::precompute(&client_public_key, &server_private_key);
    let data = sodiumoxide::crypto::box_::seal_precomputed(&data(), &nonce, &encryption_key);
    c.bench_function("box_::open_precomputed", move |b| {
        b.iter(|| {
            let _ = sodiumoxide::crypto::box_::open_precomputed(&data, &nonce, &decryption_key)
                .unwrap();
        })
    });
}

fn secret_key_encryption_bench(c: &mut Criterion) {
    sodiumoxide::init().unwrap();

    let secret_key = secretbox::gen_key();
    let nonce = secretbox::gen_nonce();
    let data = data();
    c.bench_function("secretbox::seal", move |b| {
        b.iter(|| {
            let _ = sodiumoxide::crypto::secretbox::seal(&data, &nonce, &secret_key);
        })
    });
}

fn secret_key_decryption_bench(c: &mut Criterion) {
    sodiumoxide::init().unwrap();

    let secret_key = secretbox::gen_key();
    let nonce = secretbox::gen_nonce();

    let data = sodiumoxide::crypto::secretbox::seal(&data(), &nonce, &secret_key);
    c.bench_function("secretbox::open", move |b| {
        b.iter(|| {
            let _ = sodiumoxide::crypto::secretbox::open(&data, &nonce, &secret_key).unwrap();
        })
    });
}
