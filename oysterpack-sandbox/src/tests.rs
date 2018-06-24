#[test]
fn quick_test() {
    use std::collections::HashMap;

    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);

    {
        let yellow_score = scores.entry(String::from("Yellow")).or_insert(50);
        assert_eq!(*yellow_score, 50);
    }

    {
        let blue_score = scores.entry(String::from("Blue")).or_insert(50);
        assert_eq!(*blue_score, 10);
    }

    println!("{:?}", scores);
}
