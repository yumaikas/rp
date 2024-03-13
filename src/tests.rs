
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use rust_decimal_macros::dec;
    use crate::*;

    fn new_state() -> RPState {
        return RPState {
            registers: HashMap::new(),
            stack: Vec::new(),
            eat_count: 0,
            mode: Mode::CommandChar,
            wip_str: String::from(""),
            reg_command: String::from(""),
            num: dec!(0.0),
            decimal_offset: dec!(1),
        };
    }

    #[test]
    fn test_binops() -> Result<(), Exit> {
        let mut state = new_state();

        eval("1 2+", &mut state)?;
        assert_eq!(state.stack[0], Value::Num(dec!(3)));

        eval("3 4*", &mut state)?;
        assert_eq!(state.stack[1], Value::Num(dec!(12)));

        eval("5 10/", &mut state)?;
        assert_eq!(state.stack[2], Value::Num(dec!(0.5)));

        eval("20 15-", &mut state)?;
        assert_eq!(state.stack[3], Value::Num(dec!(5)));

        eval("15 20-", &mut state)?;
        assert_eq!(state.stack[4], Value::Num(dec!(-5)));

        eval("3 2%", &mut state)?;
        assert_eq!(state.stack[5], Value::Num(dec!(1)));

        eval("2 2^", &mut state)?;
        assert_eq!(state.stack[6], Value::Num(dec!(4)));

        Ok(())
    }

    #[test]
    fn test_registers() -> Result<(), Exit> {
        let mut state = new_state();
        eval("[1+]s[inc]", &mut state)?;
        assert_eq!(state.registers["inc"], Value::Str("1+".into()));
        eval("1(inc)(inc)", &mut state)?;
        assert_eq!(state.stack[0], Value::Num(dec!(3)));



        Ok(())
    }
}
