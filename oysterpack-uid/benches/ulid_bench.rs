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

#[macro_use]
extern crate criterion;
extern crate oysterpack_uid;

use criterion::Criterion;
use oysterpack_uid::*;
use std::str::FromStr;

criterion_group!(
    ulid_benches,
    ulid_generate_bench,
    ulid_increment_bench,
    ulid_to_string_bench,
    ulid_from_str_bench,
    ulid_from_u64_tuple_bench,
    ulid_from_u128_bench,
    ulid_str_bench,
    ulid_u128_bench,
    ulid_to_bytes_bench,
    ulid_from_bytes_bench
);

criterion_group!(
    uuid_benches,
    uuid_v4_generate_bench,
    uuid_to_string_bench,
    uuid_from_str_bench
);

criterion_group!(
    typed_ulid_benches,
    typed_ulid_generate_bench,
    typed_ulid_increment_bench,
);

criterion_group!(
    ulid_serde_benches,
    ulid_serialize_bincode_bench,
    ulid_deserialize_bincode_bench,
);

criterion_main!(
    ulid_benches,
    uuid_benches,
    typed_ulid_benches,
    ulid_serde_benches
);

fn ulid_serialize_bincode_bench(c: &mut Criterion) {
    let ulid = ULID::generate();
    c.bench_function("bincode::serialize(&ULID)", move |b| {
        b.iter(|| bincode::serialize(&ulid).unwrap())
    });
}

fn ulid_deserialize_bincode_bench(c: &mut Criterion) {
    let ulid = ULID::generate();
    let ulid = bincode::serialize(&ulid).unwrap();
    c.bench_function("bincode::deserialize(&[u8])", move |b| {
        b.iter(|| {
            let _: ULID = bincode::deserialize(&ulid).unwrap();
        })
    });
}

fn ulid_to_bytes_bench(c: &mut Criterion) {
    let ulid = ULID::generate();
    c.bench_function("ULID::to_bytes", move |b| b.iter(|| ulid.to_bytes()));
}

fn ulid_from_bytes_bench(c: &mut Criterion) {
    let ulid = ULID::generate().to_bytes();
    c.bench_function("ULID::from([u8;16])", move |b| b.iter(|| ULID::from(ulid)));
}

fn ulid_generate_bench(c: &mut Criterion) {
    c.bench_function("ULID::generate", |b| b.iter(|| ULID::generate()));
}

/// ULID::increment() is about 10x faster then ULID::generate()
fn ulid_increment_bench(c: &mut Criterion) {
    let ulid = ULID::generate();
    c.bench_function("ULID::increment", move |b| b.iter(|| ulid.increment()));
}

/// ULID string encoding is about 3x faster that UUID string encoding
fn ulid_to_string_bench(c: &mut Criterion) {
    let ulid = ULID::generate();
    c.bench_function("ULID::to_string", move |b| b.iter(|| ulid.to_string()));
}

/// ULID string decoding is about 2x faster that UUID string encoding
fn ulid_from_str_bench(c: &mut Criterion) {
    let ulid = ULID::generate().to_string();
    c.bench_function("ULID::from_str", move |b| {
        b.iter(|| ULID::from_str(&ulid).unwrap())
    });
}

/// ULID (u64, u64) decoding is about 13x faster that UUID string decoding
fn ulid_from_u64_tuple_bench(c: &mut Criterion) {
    let ulid: (u64, u64) = ULID::generate().into();
    c.bench_function("ULID::from_u64_tuple", move |b| b.iter(|| ULID::from(ulid)));
}

/// ULID u128 encoding is about 3x faster that UUID string encoding
fn ulid_from_u128_bench(c: &mut Criterion) {
    let ulid: u128 = ULID::generate().into();
    c.bench_function("ULID::from_u128", move |b| b.iter(|| ULID::from(ulid)));
}

/// ULID u128 encoding is about 3x faster that UUID string encoding
fn ulid_str_bench(c: &mut Criterion) {
    c.bench_function("ulid_str", move |b| b.iter(|| oysterpack_uid::ulid_str()));
}

/// ULID u128 encoding is about 3x faster that UUID string encoding
fn ulid_u128_bench(c: &mut Criterion) {
    c.bench_function("ulid_u128", move |b| b.iter(|| oysterpack_uid::ulid_u128()));
}

/// UUID generation is about 2x faster than ULID generation
fn uuid_v4_generate_bench(c: &mut Criterion) {
    c.bench_function("Uuid::new_v4", |b| b.iter(|| uuid::Uuid::new_v4()));
}

fn uuid_to_string_bench(c: &mut Criterion) {
    let uuid = uuid::Uuid::new_v4();
    c.bench_function("Uuid::to_string", move |b| b.iter(|| uuid.to_string()));
}

/// ULID string encoding is about 3x faster that UUID string encoding
fn uuid_from_str_bench(c: &mut Criterion) {
    let uuid = uuid::Uuid::new_v4().to_string();
    c.bench_function("Uuid::from_str", move |b| {
        b.iter(|| uuid::Uuid::from_str(&uuid).unwrap())
    });
}

struct Foo;
type FooId = TypedULID<Foo>;

fn typed_ulid_generate_bench(c: &mut Criterion) {
    c.bench_function("TypedULID::generate", |b| b.iter(|| FooId::generate()));
}

/// ULID::increment() is about 10x faster then ULID::generate()
fn typed_ulid_increment_bench(c: &mut Criterion) {
    let ulid = FooId::generate();
    c.bench_function("TypedULID::increment", move |b| b.iter(|| ulid.increment()));
}
