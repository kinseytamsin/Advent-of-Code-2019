#[macro_use]
extern crate itertools;

use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::fs::File;
use std::hint::unreachable_unchecked;
use std::io::{self, prelude::*};

type IntcodeVal = usize;
type IntcodeMemState = Vec<IntcodeVal>;
type IntcodeResult<T> = Result<T, IntcodeError>;
type IntcodeValResult = IntcodeResult<IntcodeVal>;

struct IntcodeError {
    kind: ErrorKind,
}

impl Debug for IntcodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.kind, f)
    }
}

impl Display for IntcodeError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.kind.as_str())
    }
}

impl Error for IntcodeError {}

impl From<IntcodeError> for io::Error {
    fn from(e: IntcodeError) -> Self {
        Self::new(io::ErrorKind::Other, e)
    }
}

#[derive(Debug)]
enum ErrorKind {
    InvalidOpcode,
    AccessViolation
}

impl ErrorKind {
    fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::InvalidOpcode => "invalid opcode",
            ErrorKind::AccessViolation => "access violation",
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Op { Add, Mul, Exit, Unknown }

impl Op {
    fn instr_size(self) -> Option<usize> {
        match self {
            Op::Add | Op::Mul => Some(4),
            Op::Exit => Some(1),
            Op::Unknown => None,
        }
    }
}

// for converting from intcode opcode to Op
impl From<IntcodeVal> for Op {
    fn from(x: IntcodeVal) -> Self {
        match x {
            1 => Op::Add,
            2 => Op::Mul,
            99 => Op::Exit,
            _ => Op::Unknown,
        }
    }
}

struct IntcodeComputer {
    mem_state: Vec<IntcodeVal>,
}

impl IntcodeComputer {
    fn get_val(&self, addr: IntcodeVal) -> IntcodeValResult {
        match self.mem_state.get(addr) {
            Some(x) => Ok(*x),
            None => Err(IntcodeError { kind: ErrorKind::AccessViolation }),
        }
    }

    fn get_mut(&mut self, addr: IntcodeVal) -> IntcodeResult<&mut IntcodeVal> {
        match self.mem_state.get_mut(addr) {
            Some(x) => Ok(x),
            None => Err(IntcodeError { kind: ErrorKind::AccessViolation }),
        }
    }

    fn _return(&self) -> IntcodeValResult {
        self.get_val(0)
    }

    fn execute(&mut self) -> IntcodeValResult {
        let mut instr_ptr: usize = 0;
        while instr_ptr < self.mem_state.len() {
            let operation = Op::from(self.get_val(instr_ptr)?);
            match operation {
                Op::Unknown => {
                    return Err(IntcodeError {
                        kind: ErrorKind::InvalidOpcode
                    });
                },
                Op::Exit => { break; },
                _ => {
                    let exec_op = |x: IntcodeVal, y: IntcodeVal| -> IntcodeVal
                    {
                        match operation {
                            Op::Add => x + y,
                            Op::Mul => x * y,
                            // this arm will never be reached because
                            // we're already in a match arm of the same
                            // variable, and all other possible values
                            // of the enum have already been accounted
                            // for
                            _ => unsafe { unreachable_unchecked() },
                        }
                    };
                    let input1_ptr = instr_ptr + 1;
                    let input2_ptr = instr_ptr + 2;
                    let output_ptr = instr_ptr + 3;

                    let input1 = self.get_val(input1_ptr)?;
                    let input2 = self.get_val(input2_ptr)?;
                    let output = self.get_val(output_ptr)?;

                    let input1_val = self.get_val(input1)?;
                    let input2_val = self.get_val(input2)?;
                    let output_val = self.get_mut(output)?;

                    let result = exec_op(input1_val, input2_val);
                    *output_val = result;
                }
            }
            instr_ptr += operation.instr_size().unwrap();
        }
        self._return()
    }
}

impl From<Vec<IntcodeVal>> for IntcodeComputer {
    fn from(p: Vec<IntcodeVal>) -> Self {
        Self {
            mem_state: p,
        }
    }
}

fn solve_part1(p: &IntcodeMemState) -> IntcodeValResult {
    let mut program = p.to_owned();
    program[1] = 12;
    program[2] = 2;
    let mut computer = IntcodeComputer::from(program);
    computer.execute()
}

fn solve_part2(p: &IntcodeMemState) -> IntcodeValResult {
    for (noun, verb) in iproduct!(0..=99, 0..=99) {
        let mut program = p.to_owned();
        program[1] = noun;
        program[2] = verb;
        let mut computer = IntcodeComputer::from(program);
        if computer.execute()? == 19690720 { return Ok(100*noun + verb); }
    }
    // if this statement is reached, no solution was found
    panic!("No solution found for Part 2");
}

fn main() -> io::Result<()> {
    let mut f = File::open("2.txt")?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;
    buffer = buffer.trim().to_string();
    let program: IntcodeMemState = buffer.split(',')
                                         .map(|s| s.parse::<IntcodeVal>()
                                                   .unwrap())
                                         .collect();
    println!("Part 1 answer: {}", solve_part1(&program)?);
    println!("Part 2 answer: {}", solve_part2(&program)?);

    Ok(())
}
