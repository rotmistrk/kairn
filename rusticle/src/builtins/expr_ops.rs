//! Expression evaluation operators — arithmetic and comparison.

use crate::error::TclError;
use crate::value::TclValue;

pub(super) fn eval_arithmetic(left: &TclValue, op: &str, right: &TclValue) -> Result<TclValue, TclError> {
    if matches!(left, TclValue::Float(_)) || matches!(right, TclValue::Float(_)) {
        let l = left.as_float()?;
        let r = right.as_float()?;
        return Ok(TclValue::Float(match op {
            "+" => l + r,
            "-" => l - r,
            "*" => l * r,
            "/" => {
                if r == 0.0 {
                    return Err(TclError::new("divide by zero"));
                }
                l / r
            }
            "%" => {
                if r == 0.0 {
                    return Err(TclError::new("divide by zero"));
                }
                l % r
            }
            _ => return Err(TclError::new(format!("unknown operator: {op}"))),
        }));
    }
    let l = left.as_int()?;
    let r = right.as_int()?;
    Ok(TclValue::Int(match op {
        "+" => l + r,
        "-" => l - r,
        "*" => l * r,
        "/" => {
            if r == 0 {
                return Err(TclError::new("divide by zero"));
            }
            l / r
        }
        "%" => {
            if r == 0 {
                return Err(TclError::new("divide by zero"));
            }
            l % r
        }
        _ => return Err(TclError::new(format!("unknown operator: {op}"))),
    }))
}

/// Evaluate a comparison operation.
pub(super) fn eval_comparison(left: &TclValue, op: &str, right: &TclValue) -> Result<TclValue, TclError> {
    if let (Ok(l), Ok(r)) = (left.as_float(), right.as_float()) {
        return Ok(TclValue::Bool(match op {
            "==" => (l - r).abs() < f64::EPSILON,
            "!=" => (l - r).abs() >= f64::EPSILON,
            "<" => l < r,
            ">" => l > r,
            "<=" => l <= r,
            ">=" => l >= r,
            _ => return Err(TclError::new(format!("unknown operator: {op}"))),
        }));
    }
    let l = left.as_str();
    let r = right.as_str();
    Ok(TclValue::Bool(match op {
        "==" => l == r,
        "!=" => l != r,
        "<" => l < r,
        ">" => l > r,
        "<=" => l <= r,
        ">=" => l >= r,
        _ => return Err(TclError::new(format!("unknown operator: {op}"))),
    }))
}
