use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

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
    fuel_req_iter.fold(0, |acc, x| acc + x)
}

fn main() -> std::io::Result<()> {
    let f = File::open("1.txt")?;
    let buf_reader = BufReader::new(f);
    let lines_iter = buf_reader.lines();
    let masses = lines_iter.map(|l| l.unwrap().parse::<i32>().unwrap());
    let mut total_fuel = 0;
    let mut total_fuel_recursive = 0;

    for mass in masses {
        total_fuel += fuel_required(mass);
        total_fuel_recursive += fuel_required_recursive(mass);
    }

    println!("{}", total_fuel);
    println!("{}", total_fuel_recursive);

    Ok(())
}
