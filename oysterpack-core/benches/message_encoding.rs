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
#[macro_use]
extern crate oysterpack_log;
#[macro_use]
extern crate serde;

use criterion::Criterion;
use oysterpack_core::message::*;

fn encoding_benchmark(c: &mut Criterion, encoding: Encoding) {
    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
    struct Foo(String);
    impl IsMessage for Foo {
        const MESSAGE_TYPE_ID: MessageTypeId = MessageTypeId(1867384532653698871582487715619812439);
    }

    let metadata = Metadata::new(Foo::MESSAGE_TYPE_ID.message_type(), encoding, None);
    c.bench_function(&format!("encoding {:?}", encoding), move |b| b.iter(|| {
        let foo = Foo("hello 1867384532653698871582487715619812439 1867384532653698871582487715619812439 1867384532653698871582487715619812439".to_string());
        let msg = Message::new(metadata, foo);
        msg.encode().unwrap();
    }));
}

fn encoding_decoding_benchmark(c: &mut Criterion, encoding: Encoding) {
    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
    struct Foo(String);
    impl IsMessage for Foo {
        const MESSAGE_TYPE_ID: MessageTypeId = MessageTypeId(1867384532653698871582487715619812439);
    }

    let metadata = Metadata::new(Foo::MESSAGE_TYPE_ID.message_type(), encoding, None);
    c.bench_function(&format!("encoding+decoding: {:?}", encoding), move |b| b.iter(|| {
        let foo = Foo("hello 1867384532653698871582487715619812439 1867384532653698871582487715619812439 1867384532653698871582487715619812439".to_string());
        let msg = Message::new(metadata, foo);
        let msg = msg.encode().unwrap();
        msg.decode::<Foo>().unwrap();
    }));
}

// - Snappy compression is the fastest
// - lz4 cam in 2nd place, about 3x slower
// - the rest were about 4x slower than lz4
// - fastest combination was bincode + snappy
//   - message pack came in a close 2nd place
fn encoding_benchmarks(c: &mut Criterion) {
    encoding_benchmark(c, Encoding::MessagePack(None));
    encoding_benchmark(c, Encoding::MessagePack(Some(Compression::Deflate)));
    encoding_benchmark(c, Encoding::MessagePack(Some(Compression::Gzip)));
    encoding_benchmark(c, Encoding::MessagePack(Some(Compression::Zlib)));
    encoding_benchmark(c, Encoding::MessagePack(Some(Compression::Snappy)));
    encoding_benchmark(c, Encoding::MessagePack(Some(Compression::Lz4)));

    encoding_benchmark(c, Encoding::Bincode(None));
    encoding_benchmark(c, Encoding::Bincode(Some(Compression::Deflate)));
    encoding_benchmark(c, Encoding::Bincode(Some(Compression::Gzip)));
    encoding_benchmark(c, Encoding::Bincode(Some(Compression::Zlib)));
    encoding_benchmark(c, Encoding::Bincode(Some(Compression::Snappy)));
    encoding_benchmark(c, Encoding::Bincode(Some(Compression::Lz4)));

    encoding_benchmark(c, Encoding::CBOR(None));
    encoding_benchmark(c, Encoding::CBOR(Some(Compression::Deflate)));
    encoding_benchmark(c, Encoding::CBOR(Some(Compression::Gzip)));
    encoding_benchmark(c, Encoding::CBOR(Some(Compression::Zlib)));
    encoding_benchmark(c, Encoding::CBOR(Some(Compression::Snappy)));
    encoding_benchmark(c, Encoding::CBOR(Some(Compression::Lz4)));

    encoding_benchmark(c, Encoding::JSON(None));
    encoding_benchmark(c, Encoding::JSON(Some(Compression::Deflate)));
    encoding_benchmark(c, Encoding::JSON(Some(Compression::Gzip)));
    encoding_benchmark(c, Encoding::JSON(Some(Compression::Zlib)));
    encoding_benchmark(c, Encoding::JSON(Some(Compression::Snappy)));
    encoding_benchmark(c, Encoding::JSON(Some(Compression::Lz4)));
}

// - Snappy compression is the fastest
// - lz4 cam in 2nd place, about 3x slower
// - the rest were about 4x slower than lz4
// - fastest combination was bincode + snappy
//   - message pack came in a close 2nd place
fn encoding_decoding_benchmarks(c: &mut Criterion) {
    encoding_decoding_benchmark(c, Encoding::MessagePack(None));
    encoding_decoding_benchmark(c, Encoding::MessagePack(Some(Compression::Deflate)));
    encoding_decoding_benchmark(c, Encoding::MessagePack(Some(Compression::Gzip)));
    encoding_decoding_benchmark(c, Encoding::MessagePack(Some(Compression::Zlib)));
    encoding_decoding_benchmark(c, Encoding::MessagePack(Some(Compression::Snappy)));
    encoding_decoding_benchmark(c, Encoding::MessagePack(Some(Compression::Lz4)));

    encoding_decoding_benchmark(c, Encoding::Bincode(None));
    encoding_decoding_benchmark(c, Encoding::Bincode(Some(Compression::Deflate)));
    encoding_decoding_benchmark(c, Encoding::Bincode(Some(Compression::Gzip)));
    encoding_decoding_benchmark(c, Encoding::Bincode(Some(Compression::Zlib)));
    encoding_decoding_benchmark(c, Encoding::Bincode(Some(Compression::Snappy)));
    encoding_decoding_benchmark(c, Encoding::Bincode(Some(Compression::Lz4)));

    encoding_decoding_benchmark(c, Encoding::CBOR(None));
    encoding_decoding_benchmark(c, Encoding::CBOR(Some(Compression::Deflate)));
    encoding_decoding_benchmark(c, Encoding::CBOR(Some(Compression::Gzip)));
    encoding_decoding_benchmark(c, Encoding::CBOR(Some(Compression::Zlib)));
    encoding_decoding_benchmark(c, Encoding::CBOR(Some(Compression::Snappy)));
    encoding_decoding_benchmark(c, Encoding::CBOR(Some(Compression::Lz4)));

    encoding_decoding_benchmark(c, Encoding::JSON(None));
    encoding_decoding_benchmark(c, Encoding::JSON(Some(Compression::Deflate)));
    encoding_decoding_benchmark(c, Encoding::JSON(Some(Compression::Gzip)));
    encoding_decoding_benchmark(c, Encoding::JSON(Some(Compression::Zlib)));
    encoding_decoding_benchmark(c, Encoding::JSON(Some(Compression::Snappy)));
    encoding_decoding_benchmark(c, Encoding::JSON(Some(Compression::Lz4)));
}

criterion_group!(benches, encoding_benchmarks, encoding_decoding_benchmarks);

fn main() {
    benches();

    criterion::Criterion::default()
        .configure_from_args()
        .final_summary();
}
