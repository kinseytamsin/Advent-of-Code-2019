extern crate anyhow;
#[macro_use]
extern crate futures;
extern crate tokio;

use std::iter;
use std::mem;

use anyhow::{Result, Error};
use futures::{prelude::*, channel::mpsc, stream};
use tokio::{prelude::*, fs::File, io::BufReader};

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

#[tokio::main(threaded_scheduler)]
async fn main() -> Result<()> {
    let f = File::open("1.txt").await?;
    let buf_reader = BufReader::new(f);
    let lines = buf_reader.lines();
    let masses = lines.map(|line| Ok::<_, Error>(line?.parse::<i32>()?));
    let (tx, mut rx) = mpsc::channel(0);
    let (tx_r, mut rx_r) = mpsc::channel(0);
    let calculation_fut =
        masses
            .zip(
                stream::iter(iter::repeat(mpsc::Sender::clone(&tx)))
                    .zip(
                        stream::iter(iter::repeat(mpsc::Sender::clone(&tx_r)))
                    )
            )
            .map(|(x, y)| Ok::<_, Error>((x?, y)))
            .try_for_each_concurrent(
                None,
                |(mass, (mut tx, mut tx_r))| async move {
                    let f = async { fuel_required(mass) };
                    let fr = async { fuel_required_recursive(mass) };
                    let (res, res_r) =
                        join!(tx.send(f.await), tx_r.send(fr.await));
                    if let Err(e) = res {
                        Err(Error::new(e))
                    } else if let Err(e) = res_r {
                        Err(Error::new(e))
                    } else {
                        Ok(())
                    }
                },
            )
            .map_err(|e| eprintln!("Error: {}", e));
    // we never use the original Senders, only clones of them, so drop
    // the originals
    mem::drop(tx);
    mem::drop(tx_r);
    let sum_f_fut = async move {
        let mut total_fuel = 0;
        while let Some(f) = rx.next().await {
            total_fuel += f;
        }
        total_fuel
    };
    let sum_fr_fut = async move {
        let mut total_fuel_recursive = 0;
        while let Some(fr) = rx_r.next().await {
            total_fuel_recursive += fr;
        }
        total_fuel_recursive
    };
    tokio::spawn(calculation_fut);
    let (total_fuel, total_fuel_recursive) = join!(sum_f_fut, sum_fr_fut);
    println!("{}", total_fuel);
    println!("{}", total_fuel_recursive);

    Ok(())
}
