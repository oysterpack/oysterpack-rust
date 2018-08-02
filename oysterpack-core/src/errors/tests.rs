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

use super::*;
use failure;
use tests::*;

const ERR_1: ErrorId = ErrorId(1);
const ERR_2: ErrorId = ErrorId(2);
const ERR_3: ErrorId = ErrorId(3);
const ERR_4: ErrorId = ErrorId(4);
const ERR_5: ErrorId = ErrorId(5);

#[derive(Debug, Fail)]
enum ClientError {
    #[fail(display = "Err1")]
    Err1,
    #[fail(display = "Err2")]
    Err2,
    #[fail(display = "Err3")]
    Err3,
}

error_macro!(Err5, ERR_5);

macro_rules! Err1 {
    () => {
        op_error!(ERR_1, ClientError::Err1)
    };
}

macro_rules! Err2 {
    () => {
        op_error!(ERR_2, ClientError::Err2)
    };
}

macro_rules! Err3 {
    () => {
        op_error!(ERR_3, ClientError::Err3)
    };
}

#[test]
fn arc_failure_downcast_ref() {
    let err = Err3!();
    let err = ArcFailure::new(err);
    assert!(err.downcast_ref::<Error>().is_some());
    assert!(err.downcast_ref::<ArcFailure>().is_none());
}

#[test]
fn generated_error_macro() {
    run_test(|| {
        let err: Error = Err5!(ClientError::Err1);

        info!("{}", err);
        debug!("{:?}", err);

        assert_eq!(err.error_id_chain(), vec![err.id]);
    });
}

#[test]
fn simple_error() {
    run_test(|| {
        let err: Error = Err1!();

        info!("{}", err);
        debug!("{:?}", err);

        assert_eq!(err.error_id_chain(), vec![err.id]);
    });
}

#[test]
fn error_context() {
    run_test(|| {
        let err: Error = Err1!();
        let err_with_context = err.context("Some context");

        info!("err_with_context -> {}", err_with_context);
        debug!("err_with_context -> {:?}", err_with_context);

        // the context overrides the Error's Display
        assert_eq!(
            format!("{}", err_with_context),
            format!("{}", err_with_context)
        );

        {
            // the failure cause is the underlying failure
            let cause = err_with_context.cause().unwrap();
            let err: &Error = cause.downcast_ref::<Error>().unwrap();
            assert_eq!(err.id(), ERR_1);
        }

        let err = op_error!(ERR_2, err_with_context);
        info!("err -> {}", err);
        debug!("err -> {:?}", err);
        assert_eq!(err.error_id_chain(), vec![ERR_2, ERR_1]);
    });
}

#[test]
fn error_id_chain() {
    run_test(|| {
        let err: Error = Err2!();
        let err = op_error!(ERR_4, err);
        let err = op_error!(ERR_5, err);
        let err = op_error!(ERR_3, err);
        info!("error_id_chain: {}", err);
        assert_eq!(err.error_id_chain(), vec![ERR_3, ERR_5, ERR_4, ERR_2]);
    });
}
