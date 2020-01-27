#!/usr/bin/env python

class FuelReq:
    def __init__(self, curr):
        self.curr = curr
    def __iter__(self):
        return self
    def __next__(self):
        # fuel_required(x) > 0 -> x > 6
        if self.curr <= 6:
            raise StopIteration
        self.curr = fuel_required(self.curr)
        return self.curr

def fuel_required(mass): return mass//3 - 2

def fuel_required_recursive(mass):
    fuel_req_iter = FuelReq(mass)
    return sum(fuel_req_iter)

def main():
    total_fuel = 0
    total_fuel_recursive = 0
    with open('1.txt') as f:
        masses = (int(line) for line in f)
        for mass in masses:
            total_fuel += fuel_required(mass)
            total_fuel_recursive += fuel_required_recursive(mass)
    print(total_fuel)
    print(total_fuel_recursive)

if __name__ == "__main__":
    main()
