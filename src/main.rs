// use std::env; // TODO: Get args from here
use std::io::{self, BufRead};
use std::collections::HashMap;
use std::fmt;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use num_traits::pow::Pow;

mod tests;


const HELP_TEXT: &str =
    "LITERALS\n\
     0-9     integers\n\
     _       toggle sign (or, as a command, negate the top item on the stack)\n\
     [...]   string\n\
     (...)   named command (see below)\n\
     \n\
     ARITHMETIC OPERATIONS\n\
     +       add the top two items on the stack\n\
     -       subtract the top item on the stack from the second item\n\
     *       multiply the top two items on the stack\n\
     /       divide the second item on the stack by the top item\n\
     %       take the modulus of the second item by the base of the top item\n\
     ^       raise the second item to the power of the top item\n\
     v       take the square root of the top item on the stack\n\
     \n\
     STACK OPERATIONS\n\
     d       duplicate the top item on the stack\n\
     c       clear the stack\n\
     ,       drop the top item from the stack\n\
     !       something semi-implemented\n\
     \n\
     REGISTER OPERATIONS\n\
     l       load from register (register name as next character)\n\
     s       store to register (register name as next character)\n\
     \n\
     CONTROL FLOW\n\
     x       evaluate string\n\
     \n\
     META\n\
     p       print the top item of the stack, leaving it in place\n\
     n       print the top item of the stack, popping it\n\
     q       quit (you can also ^D)\n\
     ?       help (this message)\n\
     \n\
     Good luck!\n\n";

#[derive(Debug,PartialEq,Eq,Clone)]
enum Value {
        Str(String),
        Num(Decimal),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(s) => write!(f, "[{}]", s),
            Value::Num(d) => write!(f, "{}", d),
        }
    }
}


fn print_value(v: &Value) {
    match v {
        Value::Str(s) => print!("{}", s),
        Value::Num(d) => print!("{}", d),
    }
}


#[derive(Debug)]
enum Mode {
    Integer,
    Decimal,
    Str,
    CommandChar,
    CommandNamed,
    RegisterChar,
    RegisterStr,
}


#[derive(Debug)]
enum Exit {
    WithMessage(String),
    Quit,
}


fn err_msg(msg: String) -> Exit {
    Exit::WithMessage(msg)
}


fn baseline_registers() -> HashMap<String, Value> {
    let kvs = [
        ("F->C", "1! 32- 5 9/*"),
        ("C->F", "1! 9 5/* 32+"),
        ("C->K", "1! 273+"),
        ("K->C", "1! 273-"),
        ("Km->mi", "1! 1.609344/"),
        ("mi->Km", "1! 1.609344*"),
    ].map(|kv: (&str, &str)| (String::from(kv.0), Value::Str(kv.1.into())));
    HashMap::from(kvs)
}


fn main() {
    let mut state = RPState {
        registers: baseline_registers(),
        stack: Vec::new(),
        mode: Mode::CommandChar,
        wip_str: String::from(""),
        // TODO: I don't think we need a return stack
        // But this the closest thing we have right now
        reg_command: String::from(""),
        eat_count: 0,
        num: dec!(0.0),
        is_num_negative: false,
        decimal_offset: dec!(1),
    };
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match eval(&line.unwrap(), &mut state) {
            Ok(()) => {
                let mut first = true;
                print!("{}", "stack: {");
                for elem in state.stack.iter() {
                    if !first { print!("{}", ", "); }
                    print!("{}", elem);
                    first = false;
                }
                println!("{}", "}");
            },
            Err(Exit::WithMessage(msg)) => println!("Error: {}", msg),
            Err(Exit::Quit) => break,
        }
    }
}

const RADIX: u32 = 10;
const INTEGER_RADIX: Decimal = dec!(10);

fn to_decimal(input: u32) -> Decimal {
    <u32 as Into<Decimal>>::into(input)
}

fn usize_to_decimal(input: usize) -> Decimal {
    <usize as Into<Decimal>>::into(input)
}


struct RPState {
    registers: HashMap<String, Value>,
    stack: Vec<Value>,
    mode: Mode,
    reg_command: String,
    wip_str: String,
    eat_count: u32,
    is_num_negative: bool,
    num: Decimal,
    decimal_offset: Decimal
}

// TODO: Generalized shuffles
// [[a-aa\](shuf)]s[dup]
// [[ab-ba\](shuf)]s[swap]
// [[ab-b\](shuf)]s[nip]
//
// fn shuffle(_state: &mut RPState) -> Result<(), Exit> {
//    Err(err_msg("(shuf) not implemented yet!".into()))
// }

fn command_str(c: char, state: &mut RPState) -> Result<(), Exit> {
    if c != ')' {
        state.wip_str.push(c);
    } else if c == ')' {
        match state.wip_str.as_str() {
            // TODO:
            // (times)
            // (while)
            // (?) -- (cond ifTrue ifFalse)
            // [(?)x]s[if-else]
            // [[\](?)x]s[if]
            // (r?) -- show contents of registers
            // Scope out
            word => {
                if state.registers.contains_key(word) {
                    match &state.registers[word].clone() {
                        Value::Str(eval_body) => {
                            state.wip_str.clear();
                            state.mode = Mode::CommandChar;
                            return eval(&eval_body, state);
                        }
                        _ => {
                            return Err(err_msg(format!(
                                "Unable to execute ({}) as a named word",
                                word)))
                        }
                    }
                } else {
                    return Err(err_msg(format!(
                        "Unable to execute ({}) as a named word", word)))
                }
            }

        }
    }

    Ok(())
}

fn command_char(c: char, state: &mut RPState) -> Result<(), Exit> {
    if c.is_digit(RADIX) {
        state.mode = Mode::Integer;
        state.num = dec!(0);
        state.num += to_decimal(c.to_digit(10)
            .ok_or_else(|| err_msg(format!("{} isn't a digit", c)))?);
        Ok(())
    }  else if c == '_' {
        state.mode = Mode::Integer;
        state.num = dec!(0);
        state.is_num_negative = true;
        Ok(())
    } else if c == ' ' {
        // do nothing
        Ok(())
    } else if c == '(' {
        state.mode = Mode::CommandNamed;
        state.wip_str.clear();
        state.eat_count = 0;
        Ok(())
    } else if c == '[' {
        state.mode = Mode::Str;
        state.wip_str.clear();
        state.eat_count = 0;
        Ok(())
    } else if let 'p' | 'n' | 'q' | 'l' | 's' | 'v' | 'x' | 'd' | ',' | 'c'
                | '!' | '?' = c
    {
        match c {
            '?' => {
                print!("{}", HELP_TEXT);
            },
            '!' => {
                if state.stack.is_empty() {
                    return Err(err_msg("Data underflow!".into()));
                }
                match state.stack.pop() {
                    Some(Value::Num(d)) => {
                        if usize_to_decimal(state.stack.len()) < d {
                            return Err(err_msg(format!(
                                "Stack depth should be at least: {}", d)));
                        }
                    }
                    Some(Value::Str(_)) => return Err(err_msg(
                        "Cannot assert a string as a stack depth".into())),
                    None => return Err(err_msg("Data underflow!".into()))
                }
            },
            'q' => return Err(Exit::Quit),
            'c' => state.stack.clear(),
            'l' => {
                state.reg_command = "l".into();
                state.mode = Mode::RegisterChar;
            }
            'x' => {
                if state.stack.is_empty() {
                    return Err(err_msg("Data underflow!".into()));
                }
                match state.stack.pop() {
                    Some(Value::Str(s)) => {
                        return eval(&s, state)
                    }
                    Some(v) => {
                        return Err(err_msg(format!("Cannot eval {}", v)));
                    }
                    None => {
                        return Err(err_msg("Data underflow!".into()));
                    }
                }
            }
            's' => {
                state.reg_command = "s".into();
                state.mode = Mode::RegisterChar;
            }
            'd' => {
                if state.stack.is_empty() {
                    return Err(err_msg("Data underflow!".into()));
                }
                let val = state.stack.pop().unwrap();
                state.stack.push(val.clone());
                state.stack.push(val.clone());
            }
            ',' => {
                if state.stack.is_empty() {
                    return Err(err_msg("Data underflow!".into()));
                }
                state.stack.pop();
            }
            'v' => {
                if state.stack.is_empty() {
                    return Err(err_msg("Data underflow!".into()));
                }
                match state.stack.pop() {
                    Some(Value::Num(d)) => {
                        match d.sqrt() {
                            Some(n) => state.stack.push(Value::Num(n)),
                            None => return Err(err_msg(
                                "Error attempting square root!".into())),
                        }
                    }
                    Some(v) => {
                        return Err(err_msg(format!(
                            "Invalid attempt to sqrt {}", v)));
                    }
                    None => {
                        return Err(err_msg(
                            "Impossible data underflow!".into()));
                    }
                }
            }
            'p' =>  {
                if state.stack.is_empty() {
                    return Err(err_msg("Data underflow!".into()));
                }
                print_value(state.stack.last().unwrap());
            },
            'n' => {
                if state.stack.is_empty() {
                    return Err(err_msg("Data underflow!".into()));
                }
                print_value(&state.stack.pop().unwrap());
            },
            _ => return Err(err_msg(format!(
                  "{} is unimplemented, this shouldn't be reachable!", c))),
        }
        Ok(())

    } else if let '+' | '-' | '/' | '*' | '%' | '^' = c {
        if state.stack.len() < 2 {
            return Err(err_msg("Data underflow!".into()));
        }

        let a = state.stack.pop().ok_or_else(|| err_msg(
            "Data underflow!".into()))?;
        let b = state.stack.pop().ok_or_else(|| err_msg(
            "Data underflow!".into()))?;

        match (&a, &b) {
            (Value::Num(ia), Value::Num(ib)) => {
                let value = match c {
                    '+' => Some(*ib + *ia),
                    '-' => Some(*ib - *ia),
                    '/' => Some(*ib / *ia),
                    '*' => Some(*ib * *ia),
                    '%' => Some(*ib % *ia),
                    '^' => Some(Decimal::pow(*ib, *ia)),
                    _ => None
                }.ok_or_else(|| err_msg("This should never happen".into()))?;
                state.stack.push(Value::Num(value));
            }
            _ =>  {
                state.stack.push(b);
                state.stack.push(a);
                return Err(err_msg(format!(
                    "Invalid operation {} on non-numbers!", c)))
            }
        }
        Ok(())
    } else {
        panic!("{} isn't implemented yet!", c)
    }
}

fn finish_num(c: char, state: &mut RPState) -> Result<(), Exit> {
    // print!("finishing number, negative? {}", state.is_num_negative);
    if state.is_num_negative {
        state.num *= dec!(-1);
    }
    state.stack.push(Value::Num(state.num));
    state.mode = Mode::CommandChar;
    state.is_num_negative = false;
    command_char(c, state)
}

fn integer(c: char, state: &mut RPState) -> Result<(), Exit> {
    if c.is_digit(RADIX) {
        state.num *= INTEGER_RADIX;
        state.num += to_decimal(c.to_digit(10).unwrap());
    } else if c == '.' {
        state.decimal_offset = dec!(1);
        state.mode = Mode::Decimal;
    } else if c == '_' {
        state.is_num_negative = true;
    } else {
        return finish_num(c, state);
    }
    return Ok(());
}

fn decimal(c: char, state: &mut RPState) -> Result<(), Exit> {
    if c.is_digit(RADIX) {
        state.decimal_offset *= dec!(0.1);
        state.num += to_decimal(c.to_digit(10).unwrap())
                     * state.decimal_offset;
    } else {
        return finish_num(c, state)
    }
    return Ok(());
}

fn string(c: char, state: &mut RPState) -> Result<(), Exit> {
    if state.eat_count > 0 {
        state.eat_count-=1;
        state.wip_str.push(c);
    } else if c == '\\' {
        state.eat_count = 1;
    } else if c != ']' {
        state.wip_str.push(c);
    } else if c == ']' {
        state.mode = Mode::CommandChar;
        state.stack.push(Value::Str(state.wip_str.clone()));
        state.wip_str.clear();
    } else {
        return Err(err_msg("Should Not Get Here!".into()))
    }
    Ok(())
}

fn register_str(c: char, state: &mut RPState) -> Result<(), Exit> {
    match (c, state.reg_command.as_str())  {
        (']', "l")  => {
            if state.registers.contains_key(&state.wip_str) {
                state.stack.push(state.registers[&state.wip_str].clone());
            }
            state.wip_str.clear();
            state.mode = Mode::CommandChar;
        }
        (']', "s")  => {
            if state.stack.is_empty() {
                return Err(err_msg(format!(
                    "Data underflow attempting to store to register {}", c)));
            }
            state.registers.insert(state.wip_str.clone(),
                                   state.stack.pop().unwrap());
            state.wip_str.clear();
            state.mode = Mode::CommandChar;
        }
        (_, "l"|"s") => {
            state.wip_str.push(c)
        }
        _ => {
            state.mode = Mode::CommandChar;
            return Err(err_msg(format!(
                "Unsupported register command {}", state.reg_command)));
        }
    }
    Ok(())
}

fn register_char(c: char, state: &mut RPState) -> Result<(), Exit> {
    match (state.reg_command.as_str(), c) {
        (_, '[') => {
            state.mode = Mode::RegisterStr;
            Ok(())
        }
        ("s", c) => {
            if state.stack.is_empty() {
                return Err(err_msg(format!(
                    "Data underflow attempting to store to register {}", c)));
            }
            state.registers.insert(String::from(c),
                                   state.stack.pop().unwrap());
            state.mode = Mode::CommandChar;
            Ok(())
        }
        ("l", c) => {
            if state.registers.contains_key(&String::from(c)) {
                state.stack.push(state.registers[&String::from(c)].clone());
            }
            state.mode = Mode::CommandChar;
            Ok(())
        }
        _ => {
            state.mode = Mode::CommandChar;
            return Err(err_msg(format!(
                "Unsupported register command {}", state.reg_command)));
        }
    }
}

fn eval(input: &str, state: &mut RPState) -> Result<(), Exit> {
    for (_cpos, c) in input.char_indices() {
        let res = match state.mode {
            Mode::CommandChar => command_char(c, state),
            Mode::CommandNamed => command_str(c, state),
            Mode::Integer => integer(c, state),
            Mode::Decimal => decimal(c, state),
            Mode::Str => string(c, state),
            Mode::RegisterChar => register_char(c, state),
            Mode::RegisterStr => register_str(c, state),
        };
        if res.is_err() {
            return res
        }
    }
    match state.mode {
        Mode::Integer | Mode::Decimal => {
            return finish_num(' ', state)
        },
        _ => {}
    };
    return Ok(());
}
