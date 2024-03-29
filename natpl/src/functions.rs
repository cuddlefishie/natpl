use fraction::BigDecimal;

use crate::{
    num::Float,
    runtime::EvalError,
    runtime::UnitError,
    syntax::FC,
    value_unit::{Unit, Value, ValueKind},
};

macro_rules! functions {
    ($($($names:ident)|+ => $body:expr),*$(,)?) => {
        functions!(
            @names { $($($names,)*)* }
        );

        pub(crate) fn builtin_func(fc: FC, name: &str, base: &Value, args: &[(FC, Value)]) -> Option<Result<Value, EvalError>> {
            macro_rules! conv_float_fn {
                ($fun:expr) => {{
                    |n: &BigDecimal| {
                        let f = crate::num::float_from_decimal(n);
                        crate::num::decimal_from_float(&$fun(f))
                    }
                }};
            }

            macro_rules! argument_count_check {
                ($expected:expr) => {{
                    if args.len() != $expected {
                        return Some(Err(EvalError::CallArgumentMismatch {
                            fc,
                            base: base.clone(),
                            num_args_expected: $expected,
                            num_args_applied: args.len(),
                        }));
                    }
                    args
                }};
            }

            macro_rules! unary {
                ($unit_check:expr, $do_fn:expr, $unit_fn:expr) => {{
                    argument_count_check!(1);
                    let (arg_fc, arg) = &args[0];

                    if !$unit_check(&arg.unit) {
                        return Some(Err(EvalError::UnitError(UnitError::UnitMismatch { fc: *arg_fc, found: arg.unit.clone(), expected: Unit::new() })))
                    }

                    let kind = match arg.kind.map_number(&|n| $do_fn(n)) {
                        Some(kind) => kind,
                        None => return Some(Err(EvalError::ValueKindMismatch(*arg_fc, arg.clone()))),
                    };

                    let unit = $unit_fn(&arg.unit);

                    Ok(Value { kind, unit })
                }};
            }

            macro_rules! unary_unitless_float {
                ($do_fn:expr) => {
                    unary_unitless_float!($do_fn, Unit::new())
                };
                ($do_fn:expr, $unit:expr) => {
                    unary!(|unit| unit == &Unit::new(), conv_float_fn!($do_fn), |_| $unit)
                };
            }


            let res = match name {
                $($(stringify!($names))|* => $body,)*
                _ => return None,
            };

            Some(res)
        }
    };

    (@names { $($names:ident,)* }) => {
        pub(crate) static BUILTIN_FUNCTION_NAMES: &[&str] = &[
            $(stringify!($names),)*
        ];
    };
}

functions! {
    sin => unary_unitless_float!(|f: Float| f.sin()),
    asin | arcsin => unary_unitless_float!(|f: Float| f.asin()),

    cos => unary_unitless_float!(|f: Float| f.cos()),
    acos | arccos => unary_unitless_float!(|f: Float| f.acos()),

    tan => unary_unitless_float!(|f: Float| f.tan()),
    atan | arctan => unary_unitless_float!(|f: Float| f.atan()),

    sqrt => unary!(
        |_| true,
        conv_float_fn!(|f: Float| f.sqrt()),
        |u: &Unit| u.pow(&(BigDecimal::from(1) / BigDecimal::from(2)))
    ),

    cbrt => unary!(
        |_| true,
        conv_float_fn!(|f: Float| f.cbrt()),
        |u: &Unit| u.pow(&(BigDecimal::from(1) / BigDecimal::from(3)))
    ),

    log | log10 => unary_unitless_float!(|f: Float| f.log10()),
    log2 => unary_unitless_float!(|f: Float| f.log2()),
    ln => unary_unitless_float!(|f: Float| f.ln()),

    exp => unary_unitless_float!(|f: Float| f.exp()),

    ceil => unary_unitless_float!(|f: Float| f.ceil()),
    floor => unary_unitless_float!(|f: Float| f.floor()),
    round => unary_unitless_float!(|f: Float| f.round()),

    len => {
        let args = argument_count_check!(1);
        let (arg_fc, arg) = &args[0];

        match &arg.kind {
            ValueKind::Vector(e) => {
                Ok(Value {
                    kind: ValueKind::Number(e.len().into()),
                    unit: Unit::new()
                })
            }
            _ => {
                Err(EvalError::ValueKindMismatch(*arg_fc, arg.clone()))
            }
        }
    }
}
