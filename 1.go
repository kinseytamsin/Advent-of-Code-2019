package main

import (
    "bufio"
    "fmt"
    "os"
    "strconv"
)

type FuelReq struct {
    curr int
}

func (f *FuelReq) next() (int, bool) {
    // fuelRequired(x) > 0 -> x > 6
    if f.curr <= 6 {
        return 0, false
    } else {
        f.curr = fuelRequired(f.curr)
        return f.curr, true
    }
}

func eprintln(a ...interface{}) (n int, err error) {
    return fmt.Fprintln(os.Stderr, a...)
}

func fuelRequired(mass int) int {
    return mass/3 - 2
}

func fuelRequiredRecursive(mass int) (f int) {
    fuelReqIter := FuelReq{mass}
    more := true
    var curr int
    for more {
        curr, more = fuelReqIter.next()
        f += curr
    }
    return
}

func main() {
    inputFile, err := os.Open("1.txt")
    if err == nil {
        defer inputFile.Close()
    } else {
        eprintln("Error opening input file:", err)
    }
    scanner := bufio.NewScanner(inputFile)
    var totalFuel, totalFuelRecursive int
    for scanner.Scan() {
        line := scanner.Text()
        mass, err := strconv.Atoi(line)
        if err != nil {
            eprintln(err)
        }
        totalFuel += fuelRequired(mass)
        totalFuelRecursive += fuelRequiredRecursive(mass)
    }
    if err := scanner.Err(); err != nil {
        eprintln(err)
    }
    fmt.Println(totalFuel)
    fmt.Println(totalFuelRecursive)
}
