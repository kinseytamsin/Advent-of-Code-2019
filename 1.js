#!/usr/bin/env node
"use strict";
/* jshint esversion: 6 */

const fs = require("fs");
const readline = require("readline");
const stream = require("stream");
const Transform = stream.Transform || require("readable-stream").Transform;

class FuelReq {
    constructor(curr) {
        this.curr = curr;
    }
    [Symbol.iterator]() {
        return this;
    }
    next() {
        // fuelRequired(x) > 0 -> x > 6
        if (this.curr <= 6) {
            return { done: true };
        } else {
            this.curr = fuelRequired(this.curr);
            return { value: this.curr };
        }
    }
}

function fuelRequiredRecursive(mass) {
    let fuelReqIter = new FuelReq(mass);
    let result = 0;
    for (let i of fuelReqIter) {
        result += i;
    }
    return result;
}

function fuelRequired(mass) {
    return Math.floor(mass / 3) - 2;
}

const resultStream = new Transform({
    transform(line, enc, cb) {
        let mass = parseInt(line);
        let calculate = Promise.all([
            Promise.resolve(fuelRequired(mass)),
            Promise.resolve(fuelRequiredRecursive(mass))
        ]);
        calculate.then((results) => {
            this.push({
                "fuel": results[0],
                "fuelRecursive": results[1]
            });
        });
        cb();
    },
    readableObjectMode: true
});
const rl = readline.createInterface({
    input: fs.createReadStream("1.txt")
});

rl.on("line", (line) => {
    resultStream.write(line);
});

let totalFuel = 0;
let totalFuelRecursive = 0;

resultStream.on("data", (result) => {
    Promise.all([
        new Promise((resolve, reject) => {
            totalFuel += result.fuel;
            resolve();
        }),
        new Promise((resolve, reject) => {
            totalFuelRecursive += result.fuelRecursive;
            resolve();
        })
    ]);
});

rl.on("close", () => {
    console.log("%d", totalFuel);
    console.log("%d", totalFuelRecursive);
});
