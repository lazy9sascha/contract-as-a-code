use caac::batch::{BatchProcessor, BatchResult, ItemResult, run};
use serde_json::{Value, json};
use std::cell::RefCell;

fn pair(numerator: i64, denominator: i64) -> Value {
    json!({"numerator": numerator, "denominator": denominator})
}

struct RecordingProcessor {
    calls: RefCell<Vec<(i64, i64)>>,
}

impl BatchProcessor for RecordingProcessor {
    fn apply(&self, numerator: i64, denominator: i64) -> ItemResult {
        self.calls.borrow_mut().push((numerator, denominator));
        if denominator == 0 {
            ItemResult::ZeroDivisor
        } else {
            ItemResult::Ok(numerator / denominator)
        }
    }
}

#[test]
fn processor_validates_before_apply_and_calls_once_in_order() {
    let processor = RecordingProcessor {
        calls: RefCell::new(Vec::new()),
    };

    assert_eq!(
        processor.run(&json!([pair(4, 2), pair(1001, 1)])),
        BatchResult::InvalidInput
    );
    assert!(processor.calls.borrow().is_empty());

    assert_eq!(
        processor.run(&json!([pair(1, 0), pair(9, 2), pair(9, 2)])),
        BatchResult::Completed(vec![
            ItemResult::ZeroDivisor,
            ItemResult::Ok(4),
            ItemResult::Ok(4),
        ])
    );
    assert_eq!(*processor.calls.borrow(), vec![(1, 0), (9, 2), (9, 2)]);
}

#[test]
fn signed_division_and_boundaries() {
    let cases = [
        (7, 2, 3),
        (-7, 2, -3),
        (7, -2, -3),
        (-7, -2, 3),
        (0, -2, 0),
        (-1000, 1, -1000),
        (1000, 1, 1000),
        (1, -1000, 0),
        (1000, 1000, 1),
    ];

    for (numerator, denominator, expected) in cases {
        assert_eq!(
            run(&json!([pair(numerator, denominator)])),
            BatchResult::Completed(vec![ItemResult::Ok(expected)]),
            "division of {numerator} by {denominator}"
        );
    }
}

#[test]
fn empty_and_repeated_runs_return_independent_values() {
    assert_eq!(run(&json!([])), BatchResult::Completed(vec![]));

    let first = run(&json!([pair(4, 2)]));
    let second = run(&json!([pair(9, 2)]));

    assert_eq!(first, BatchResult::Completed(vec![ItemResult::Ok(2)]));
    assert_eq!(second, BatchResult::Completed(vec![ItemResult::Ok(4)]));
    assert_ne!(first, second);
}

#[test]
fn results_preserve_order_duplicates_and_continue_after_zero_divisors() {
    let input = json!([pair(1, 0), pair(9, 2), pair(9, 2)]);
    let original = input.clone();

    assert_eq!(
        run(&input),
        BatchResult::Completed(vec![
            ItemResult::ZeroDivisor,
            ItemResult::Ok(4),
            ItemResult::Ok(4),
        ])
    );
    assert_eq!(input, original);
}

#[test]
fn invalid_entire_input_is_rejected() {
    let mut invalid_values = vec![
        Value::Null,
        json!(42),
        json!("items"),
        json!({"items": [pair(1, 1)]}),
        json!([null]),
        json!([42]),
        json!([[]]),
        json!([{}]),
        json!([{"numerator": 1}]),
        json!([{"denominator": 1}]),
        json!([{"numerator": 1, "denominator": 1, "extra": 0}]),
        json!([{"numerator": true, "denominator": 1}]),
        json!([{"numerator": 1, "denominator": false}]),
        json!([{"numerator": 1.0, "denominator": 1}]),
        json!([{"numerator": 1, "denominator": 1.0}]),
        json!([{"numerator": "1", "denominator": 1}]),
        json!([{"numerator": u64::MAX, "denominator": 1}]),
        json!([{"numerator": 1, "denominator": i64::MIN}]),
        json!([pair(1001, 1)]),
        json!([pair(-1001, 1)]),
        json!([pair(1, 1001)]),
        json!([pair(1, -1001)]),
        json!([pair(1, 1), pair(1001, 1)]),
    ];
    invalid_values.push(Value::Array((0..101).map(|_| pair(1, 1)).collect()));

    for input in invalid_values {
        assert_eq!(run(&input), BatchResult::InvalidInput, "input: {input}");
    }
}

#[test]
fn maximum_batch_length_is_valid() {
    let input = Value::Array((0..100).map(|_| pair(1, 1)).collect());
    assert_eq!(
        run(&input),
        BatchResult::Completed(vec![ItemResult::Ok(1); 100])
    );
}

#[test]
fn division_matches_an_independent_repeated_subtraction_oracle() {
    for numerator in -12_i64..=12 {
        for denominator in -12_i64..=12 {
            let expected = if denominator == 0 {
                ItemResult::ZeroDivisor
            } else {
                let mut remainder = numerator.abs();
                let divisor = denominator.abs();
                let mut quotient = 0;
                while remainder >= divisor {
                    remainder -= divisor;
                    quotient += 1;
                }
                if (numerator < 0) ^ (denominator < 0) {
                    ItemResult::Ok(-quotient)
                } else {
                    ItemResult::Ok(quotient)
                }
            };

            assert_eq!(
                run(&json!([pair(numerator, denominator)])),
                BatchResult::Completed(vec![expected]),
                "division of {numerator} by {denominator}"
            );
        }
    }
}
