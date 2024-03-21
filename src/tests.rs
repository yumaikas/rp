
#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use crate::*;

    fn new_state() -> RPState {
        return RPState {
            registers: baseline_regsiters(),
            stack: Vec::new(),
            eat_count: 0,
            mode: Mode::CommandChar,
            is_num_negative: false,
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

        eval("c_40 32-", &mut state)?;
        assert_eq!(state.stack[0], Value::Num(dec!(-72)));

        Ok(())
    }

    #[test]
    fn test_registers() -> Result<(), Exit> {
        let mut state = new_state();
        eval("[1+]s[inc]", &mut state)?;
        assert_eq!(state.registers["inc"], Value::Str("1+".into()));
        eval("1(inc)(inc)", &mut state)?;
        assert_eq!(state.stack[0], Value::Num(dec!(3)));

        eval("c1(inc)", &mut state)?;
        assert_eq!(state.stack[0], Value::Num(dec!(2)));

        Ok(())
    }

    #[test]
    fn test_stdlib() -> Result<(), Exit> {
        let mut state = new_state();
        // TODO: Add this back in one we've figured out booleans
        // comparison
        //
        // Right now, `5 9 /` results in 0.5555555555555555555555555556
        // Which means that this conversion results in 
        // -40.000000000000000000000000003
        // Which, like, decimal bases are fun and all
        //
        // And I don't want to fuss with unwrapping state and all that
        // mess. I'd much rather do something like 
        // eval("_40(F->C)", &mut state)?;
        // assert_eq!(state.stack[0], Value::Num(dec!(-40)));

        eval("_40(C->F)", &mut state)?;
        assert_eq!(state.stack[0], Value::Num(dec!(-40)));

        eval("c0(C->F)", &mut state)?;
        assert_eq!(state.stack[0], Value::Num(dec!(32)));

        Ok(())
    }
}
