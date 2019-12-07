#include <stdio.h>
#include <stdlib.h>

int fuel_required(int mass) {
    return mass/3 - 2;
}

int fuel_required_recursive(int mass) {
    int total = 0;
    int curr = mass;
    while (curr > 6) {
    /* fuel_required(x) > 0 -> x > 6 */
        curr = fuel_required(curr);
        total += curr;
    }
    return total;
}

int main(void) {
    FILE *fp = fopen("1.txt", "r");
    int total_fuel = 0;
    int total_fuel_recursive = 0;
    int mass;

    while (fscanf(fp, "%d", &mass) != EOF) {
        total_fuel += fuel_required(mass);
        total_fuel_recursive += fuel_required_recursive(mass);
    }

    printf("%d\n", total_fuel);
    printf("%d\n", total_fuel_recursive);
    return(EXIT_SUCCESS);
}
