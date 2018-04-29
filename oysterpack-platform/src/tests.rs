// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! tests

use super::*;

#[test]
fn domain_name_get() {
    let domain_name = "   OysterPack   ";
    let name = DomainName::new(domain_name).unwrap();
    assert_eq!(name.get(), &domain_name.trim().to_lowercase());
}

#[test]
fn blank_domain_name() {
    if let Err(err @ NameError::TooShort { .. }) = DomainName::new("       ") {
        assert!(format!("{}", err).starts_with("Name min length is "));
    } else {
        panic!("NameError::TooShort error should have been returned")
    }
}

#[test]
fn name_too_short() {
    if let Err(err @ NameError::TooShort { .. }) = DomainName::new("   12   ") {
        assert!(format!("{}", err).starts_with("Name min length is "));
        if let NameError::TooShort { len, .. } = err {
            assert_eq!(2, len);
        }
    } else {
        panic!("NameError::NameTooShort error should have been returned")
    }
}

#[test]
fn name_too_long() {
    let name = vec!['a'; 65];
    let name = name.iter().fold(("".to_string(), 0), |mut s, c| {
        s.0.insert(s.1, *c);
        s.1 += 1;
        s
    });

    if let Err(err @ NameError::TooLong { .. }) = DomainName::new(&name.0) {
        assert!(format!("{}", err).starts_with("Name max length is "));
        if let NameError::TooLong { len, .. } = err {
            assert_eq!(65, len);
        }
    } else {
        panic!("NameError::TooLong error should have been returned")
    }
}

#[test]
fn valid_names() {
    // min length = 3
    let name = DomainName::new("aBc").unwrap();
    assert_eq!("abc", name.get());
    // alphanumeric and _ are allowed
    let name = DomainName::new("abc_DEF_123-456").unwrap();
    assert_eq!("abc_def_123-456", name.get());

    // max length = 64
    let name = vec!['a'; 64];
    let name = name.iter().fold(("".to_string(), 0), |mut s, c| {
        s.0.insert(s.1, *c);
        s.1 += 1;
        s
    });
    assert_eq!(64, name.0.len());
    DomainName::new(&name.0).unwrap();
}

#[test]
fn invalid_names() {
    match DomainName::new("aB c") {
        Err(NameError::Invalid { name }) => assert_eq!("ab c", &name),
        other => panic!(
            "NameError::Invalid error should have been returned, but instead received : {:?}",
            other
        ),
    }

    match DomainName::new("-abc") {
        Err(NameError::StartsWithNonAlpha { name }) => assert_eq!("-abc", &name),
        other => panic!("NameError::StartsWithNonAlpha error should have been returned, but instead received : {:?}", other)
    }

    match DomainName::new("_abc") {
        Err(NameError::StartsWithNonAlpha { name }) => assert_eq!("_abc", &name),
        other => panic!("NameError::StartsWithNonAlpha error should have been returned, but instead received : {:?}", other)
    }

    match DomainName::new("1abc") {
        Err(NameError::StartsWithNonAlpha { name }) => assert_eq!("1abc", &name),
        other => panic!("NameError::StartsWithNonAlpha error should have been returned, but instead received : {:?}", other)
    }
}

#[test]
fn app_instance_is_send() {
    struct Foo<T: Send>(T);

    let mut services = HashSet::new();
    services.insert(Service::new(
        ServiceId::new(),
        ServiceName("service-1".to_string()),
        Version::parse("1.2.3").unwrap(),
    ));
    services.insert(Service::new(
        ServiceId::new(),
        ServiceName("service-2".to_string()),
        Version::parse("2.2.3").unwrap(),
    ));
    let app = App::new(
        DomainId::new(),
        AppId::new(),
        AppName("app".to_string()),
        Version::parse("1.2.3").unwrap(),
        services,
    ).unwrap();
    let app_instance = AppInstance::new(app, AppInstanceId::new());

    let _ = Foo(app_instance);
}
