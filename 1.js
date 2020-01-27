const fs = require('fs');
const readline = require('readline');
const stream = require('stream');
const Transform = stream.Transform || require('readable-stream').Transform;

function FuelReq(curr) {
    this.curr = curr;
}

FuelReq.prototype[Symbol.iterator] = function() {
    return this;
};

FuelReq.prototype.next = function() {
    // fuelRequired(x) > 0 -> x > 6
    if (this.curr <= 6) {
        return { done: true };
    } else {
        this.curr = fuelRequired(this.curr);
        return { value: this.curr };
    }
};

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
    transform: function(line, enc, cb) {
        let mass = parseInt(line);
        this.push({
            'fuel': fuelRequired(mass),
            'fuelRecursive': fuelRequiredRecursive(mass)
        });
        cb();
    },
    readableObjectMode: true
});
const rl = readline.createInterface({
    input: fs.createReadStream('1.txt')
});

rl.on('line', (line) => {
    resultStream.write(line);
});

let totalFuel = 0;
let totalFuelRecursive = 0;

resultStream.on('data', (result) => {
    totalFuel += result.fuel;
    totalFuelRecursive += result.fuelRecursive;
});

rl.on('close', () => {
    console.log('%d', totalFuel);
    console.log('%d', totalFuelRecursive);
});
