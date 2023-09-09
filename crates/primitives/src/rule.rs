//! This module contains the [Rule] type as well as several helper macros for applying
//! rules on top of one another.

type Rule<T> = Box<dyn Fn(T) -> anyhow::Result<T>>;

#[macro_export]
macro_rules! chain_rules {
    ($state:expr, $($rule:expr),+) => {{
        let mut result = Ok($state);

        $(
            result = match result {
                Ok(val) => $rule(val),
                err @ Err(_) => err,
            };
        )+

        result
    }};
}

mod test {
    use super::*;

    #[test]
    fn apply_sequential_rules() {
        let state = 5;

        let rule_lt_10: Rule<u32> = Box::new(|state: u32| {
            if state < 10 {
                Ok(state)
            } else {
                Err(anyhow::anyhow!("state must be less than 10"))
            }
        });
        let rule_double_10: Rule<u32> = Box::new(|state: u32| {
            if state * 2 == 10 {
                Ok(state)
            } else {
                Err(anyhow::anyhow!("state must be half of 10"))
            }
        });
        let rule_bitwise: Rule<u32> = Box::new(|state: u32| {
            if state & 0xF == 0b0101 {
                Ok(state)
            } else {
                Err(anyhow::anyhow!("state must be 5"))
            }
        });

        let result = chain_rules!(state, rule_lt_10, rule_double_10, rule_bitwise);
        assert!(result.is_ok());
    }

    #[test]
    fn fail_sequential_rules() {
        let state = 5;

        let rule_lt_10: Rule<u32> = Box::new(|state: u32| {
            if state < 10 {
                Ok(state)
            } else {
                Err(anyhow::anyhow!("state must be less than 10"))
            }
        });
        let rule_double_11: Rule<u32> = Box::new(|state: u32| {
            if state * 2 == 11 {
                Ok(state)
            } else {
                Err(anyhow::anyhow!("state must be half of 11"))
            }
        });
        let rule_bitwise: Rule<u32> = Box::new(|state: u32| {
            if state & 0xF == 0b0101 {
                Ok(state)
            } else {
                Err(anyhow::anyhow!("state must be 5"))
            }
        });

        let result = chain_rules!(state, rule_lt_10, rule_double_11, rule_bitwise);
        assert!(result.is_err());
    }
}
