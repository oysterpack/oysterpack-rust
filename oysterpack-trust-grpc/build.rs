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

use protobuf_codegen::Customize;

fn main() {
    compile_grpc_protos();
    compile_test_grpc_protos();
    compile_bench_grpc_protos();
}

fn compile_test_grpc_protos() {
    protoc(
        vec!["message.proto", "foo.proto"],
        vec!["tests/protos"],
        "tests/protos",
        None,
    );
}

fn compile_bench_grpc_protos() {
    protoc(
        vec!["foo.proto"],
        vec!["benches/protos"],
        "benches/protos",
        None,
    );
}

fn compile_grpc_protos() {
    protoc(
        vec!["message.proto", "metrics.proto"],
        vec!["protos"],
        "src/protos",
        None,
    );
}

pub fn protoc(
    inputs: Vec<&str>,
    includes: Vec<&str>,
    output: &str,
    customizations: Option<Customize>,
) {
    for dir in includes.iter() {
        println!("cargo:rerun-if-changed={}", dir);
        println!("cargo:rerun-if-changed={}/**", dir);
    }

    protoc_grpcio::compile_grpc_protos(inputs, includes, output, customizations)
        .expect("Failed to compile gRPC definitions!");
}
