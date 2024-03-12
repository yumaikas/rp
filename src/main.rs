// use std::env; // TODO: Get args from here
use std::collections::HashMap;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

#[derive(Debug)]
enum Value {
        Str(String),
        Num(Decimal),
}

enum Mode {
    Integer,
    Decimal,
    Str,
    Command
}


fn main() {
    let mut state = RPState {
        registers: HashMap::new(),
        stack: Vec::new(),
        mode: Mode::Command,
        wip_str: String::from(""),
        num: dec!(0.0),
        decimal_offset: dec!(1),
    };

    let result = eval(String::from("123 456 7.89+"), &mut state);
    match result {
        Ok(()) => println!("{:?}", state.stack),
        Err(msg) => println!("Error: {}", msg),
    }
}

const RADIX: u32 = 10;
const INTEGER_RADIX: Decimal = dec!(10);

fn to_decimal(input: u32) -> Decimal {
    <u32 as Into<Decimal>>::into(input)
}


struct RPState {
    registers: HashMap<String, Value>,
    stack: Vec<Value>,
    mode: Mode,
    wip_str: String,
    num: Decimal,
    decimal_offset: Decimal
}

fn command(c: char, state: &mut RPState) -> Result<(), String> { 
    if c.is_digit(RADIX) {
        state.mode = Mode::Integer;
        state.num = dec!(0);
        state.num += to_decimal(c.to_digit(10).ok_or_else(|| format!("{} isn't a digit", c))?);
        Ok(())
    } else if c == ' ' {
        // do nothing
        Ok(())
    } else if c == '+' {
        
        let a = state.stack.pop().ok_or_else(|| "Data underflow!")?;
        let b = state.stack.pop().ok_or_else(|| "Data underflow!")?;

        match (a, b) {
            (Value::Num(c), Value::Num(d)) => { 
                state.stack.push(Value::Num(c + d));
            }

            _ => return Err("Invalid operation + on non-numbers!".to_string())
        }
        Ok(())
    } else {
        panic!("{} isn't implemented yet!", c)
    }
}

fn integer(c: char, state: &mut RPState) -> Result<(), String> {
    if c.is_digit(RADIX) {
        state.num *= INTEGER_RADIX;
        state.num += to_decimal(c.to_digit(10).unwrap());
    } else if c == '.' {
        state.decimal_offset = dec!(1);
        state.mode = Mode::Decimal;
    }
    else {
        state.stack.push(Value::Num(state.num));
        state.mode = Mode::Command;
        return command(c, state);
    }
    return Ok(());
}

fn decimal(c: char, state: &mut RPState) -> Result<(), String> {
    if c.is_digit(RADIX) {
        state.decimal_offset *= dec!(0.1);
        state.num += to_decimal(c.to_digit(10).unwrap()) * state.decimal_offset;
    } else {
        state.stack.push(Value::Num(state.num));
        state.mode = Mode::Command;
        return command(c, state);
    }
    return Ok(());
}

fn eval(input: String, state: &mut RPState) -> Result<(), String> {
    for (_cpos, c) in input.char_indices() {
        let res = match state.mode {
            Mode::Command => command(c, state),
            Mode::Integer => integer(c, state),
            Mode::Decimal => decimal(c, state),
            Mode::Str => {
                panic!("Strings not implemented yet");
            }
        };
        if res.is_err() {
            return res
        }
    }
    match state.mode {
        Mode::Integer | Mode::Decimal => { 
            state.stack.push(Value::Num(state.num));
            state.mode = Mode::Command;
        },
        _ => {}
    };
    return Ok(());
}
