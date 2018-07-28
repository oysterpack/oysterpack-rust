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
use tests;

#[test]
fn macro_src_loc() {
    tests::run_test(|| {
        let src_loc: SourceCodeLocation = src_loc!();
        debug!("src_loc = '{:?}'", src_loc);
        assert_eq!(src_loc.module_path(), "oysterpack_core::devops::tests");
        assert_eq!(src_loc.crate_name(), "oysterpack_core");
        assert_eq!(src_loc.line(), 21);

        let src_loc = foo::src_loc();
        debug!("src_loc = '{:?}'", src_loc);
        info!("src_loc = '{}'", src_loc);
        assert_eq!(src_loc.module_path(), "oysterpack_core::devops::tests::foo");
        assert_eq!(src_loc.crate_name(), "oysterpack_core");
        assert_eq!(src_loc.line(), 40);
    });
}

mod foo {
    use devops::SourceCodeLocation;

    pub(crate) fn src_loc() -> SourceCodeLocation {
        src_loc!()
    }
}
