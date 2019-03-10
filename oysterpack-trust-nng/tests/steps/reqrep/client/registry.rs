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

use cucumber_rust::*;

use crate::steps::reqrep::client::{register_basic_client, try_register_basic_client, World};
use oysterpack_trust::concurrent::messaging::reqrep::ReqRepId;
use oysterpack_trust_nng::reqrep::client as nng_client;
use oysterpack_trust_nng::reqrep::client::ClientRegistrationError;

steps!(World => {
    // [01D5J1EQ3W3M40892VXBKSYY0Q] ReqRep clients are globally registered using ReqRepId as the registry key

    // Scenario: [01D5J1HJMGKN7AF39DPP4TBYRE] Register a ReqRep service using a unique ReqRepId
    then regex "01D5J1HJMGKN7AF39DPP4TBYRE" | world, _matches, _step | {
        let reqrep_id = ReqRepId::generate();
        world.reqrep_id = Some(reqrep_id);
        let client = register_basic_client(reqrep_id);
        assert_eq!(client.id(), reqrep_id);
    };

    then regex "01D5J1HJMGKN7AF39DPP4TBYRE" | world, _matches, _step | {
        world.reqrep_id.iter().cloned().for_each(|id| {
            let client = nng_client::client(id).unwrap();
            assert_eq!(client.id(), id);
        })
    };

    // Scenario: [01D5J244J52Y4A7WGZ67ZNP0RS] Try to register 2 ReqRep services using the same ReqRepId
    given regex "01D5J244J52Y4A7WGZ67ZNP0RS" | world, _matches, _step | {
        let reqrep_id = ReqRepId::generate();
        world.reqrep_id = Some(reqrep_id);
        let client = register_basic_client(reqrep_id);
        assert_eq!(client.id(), reqrep_id);
    };

    then regex "01D5J244J52Y4A7WGZ67ZNP0RS" | world, _matches, _step | {
        world.reqrep_id.iter().cloned().for_each(|id| {
            match try_register_basic_client(id) {
                Ok(_) => panic!("Should have failed to register"),
                Err(ClientRegistrationError::ClientAlreadyRegistered(reqrep_id)) => assert_eq!(reqrep_id, id),
                Err(err) => panic!(format!("Failed with unexpected error: {}", err))
            }
        })
    };

});
