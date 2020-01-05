extern crate rayon;

use std::fs;
use std::io;

use rayon::prelude::*;

struct FuelReq { curr: i32 }

impl FuelReq {
    fn new(curr: i32) -> FuelReq {
        FuelReq { curr }
    }
}

impl Iterator for FuelReq {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        // fuel_required(x) > 0 -> x > 6
        if self.curr <= 6 {
            None
        } else {
            self.curr = fuel_required(self.curr);
            Some(self.curr)
        }
    }
}

fn fuel_required(mass: i32) -> i32 { mass/3 - 2 }

fn fuel_required_recursive(mass: i32) -> i32 {
    let fuel_req_iter = FuelReq::new(mass);
    fuel_req_iter.sum()
}

fn main() -> io::Result<()> {
    let contents = fs::read_to_string("1.txt")?;
    let lines = contents.par_lines();
    let masses = lines.map(|l| l.parse::<i32>().unwrap());
    let (total_fuel, total_fuel_recursive) = masses
        .fold(
            || (0, 0),
            |(acc_f, acc_fr), mass| (
                acc_f + fuel_required(mass),
                acc_fr + fuel_required_recursive(mass),
            ),
        )
        .reduce(
            || (0, 0),
            |(acc_f, acc_fr), (f, fr)| (acc_f + f, acc_fr + fr),
        );

    println!("{}", total_fuel);
    println!("{}", total_fuel_recursive);

    Ok(())
}
