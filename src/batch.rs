use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatchResult {
    InvalidInput,
    Completed(Vec<ItemResult>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemResult {
    Ok(i64),
    ZeroDivisor,
}

/// A concrete implementation of the batch contract's `apply` extension point.
pub trait BatchProcessor {
    fn apply(&self, numerator: i64, denominator: i64) -> ItemResult;

    fn run(&self, input: &Value) -> BatchResult {
        let Some(items) = input.as_array() else {
            return BatchResult::InvalidInput;
        };
        if items.len() > 100 {
            return BatchResult::InvalidInput;
        }

        let mut pairs = Vec::with_capacity(items.len());
        for item in items {
            let Some(fields) = item.as_object() else {
                return BatchResult::InvalidInput;
            };
            if fields.len() != 2 {
                return BatchResult::InvalidInput;
            }
            let Some(numerator) = fields.get("numerator").and_then(Value::as_i64) else {
                return BatchResult::InvalidInput;
            };
            let Some(denominator) = fields.get("denominator").and_then(Value::as_i64) else {
                return BatchResult::InvalidInput;
            };
            if !(-1000..=1000).contains(&numerator) || !(-1000..=1000).contains(&denominator) {
                return BatchResult::InvalidInput;
            }
            pairs.push((numerator, denominator));
        }

        BatchResult::Completed(
            pairs
                .into_iter()
                .map(|(numerator, denominator)| self.apply(numerator, denominator))
                .collect(),
        )
    }
}

/// The concrete signed integer division profile.
pub struct Divide;

impl BatchProcessor for Divide {
    fn apply(&self, numerator: i64, denominator: i64) -> ItemResult {
        if denominator == 0 {
            ItemResult::ZeroDivisor
        } else {
            ItemResult::Ok(numerator / denominator)
        }
    }
}

/// Evaluate a JSON batch of integer division pairs.
///
/// `Value::Null` represents absent input. The function validates the complete
/// batch before it evaluates any item.
pub fn run(input: &Value) -> BatchResult {
    let processor = Divide;
    processor.run(input)
}
