#define _ISO_C99_SOURCE

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

bool is_matching_number(int number) {
    int remaining_digits = number;
    int last_digit = -1;
    int curr_digit = -1;
    int identical_digit_pairs = 0;
    while (remaining_digits) {
        if (curr_digit != -1) last_digit = curr_digit;
        curr_digit = remaining_digits % 10;
        remaining_digits /= 10;
        if (last_digit == -1) continue;
        if (curr_digit == last_digit) identical_digit_pairs++;
        /* need to check for strictly *decreasing* digits because we're
         * going through the digits backwards
         */
        if (curr_digit > last_digit) {
            return false;
        }
    }
    if (identical_digit_pairs > 0) {
        return true;
    } else {
        return false;
    }
}

int count_matching_numbers(int min, int max, bool (*matching_func_ptr)(int)) {
    int matching_numbers = 0;
    for (int i = min; i <= max; i++) {
        if ((*matching_func_ptr)(i)) matching_numbers++;
    }
    return matching_numbers;
}

int main(void) {
    int min = 357253;
    int max = 892942;
    printf("%d\n", count_matching_numbers(min, max, &is_matching_number));
}
