#include <stdio.h>
#include <stdlib.h>
#include <stddef.h>
#include <string.h>

#define DECLARE_SUM_REDUCE_UNIVAR_INT_MAP_FUNC(FUNC_NAME, MAP_FUNC_NAME) \
    int FUNC_NAME(const int arr[], size_t n) {                           \
        int map_arr[n];                                                  \
        for (int i = 0; i < n; i++) {                                    \
            map_arr[i] = MAP_FUNC_NAME(arr[i]);                          \
        }                                                                \
        return sum_reduce(map_arr, n);                                   \
    }

typedef struct StrLink StrLink;

struct StrLink {
    StrLink *next;
    char *data;
};

void str_link_append(StrLink **head_ptr, char *new_data) {
    StrLink *new_link = malloc(sizeof(StrLink));
    StrLink *last = *head_ptr;

    new_link->data = new_data;
    new_link->next = NULL;

    if (*head_ptr == NULL) {
        *head_ptr = new_link;
        return;
    }

    while (last->next != NULL) {
        last = last->next;
    }

    last->next = new_link;
    return;
}

void free_str_linked_list(StrLink **head_ptr) {
    StrLink *tmp;

    while (*head_ptr != NULL) {
        tmp = *head_ptr;
        *head_ptr = (*head_ptr)->next;
        free(tmp->data);
        free(tmp);
    }
    *head_ptr = NULL;
}

size_t str_linked_list_len(StrLink *head) {
    size_t ret = 0;

    if (head == NULL) return 0;
    {
        StrLink *iter = head;
        do {
            ret++;
        } while ((iter = iter->next) != NULL);
    }
    return ret;
}

void readlines(StrLink **lines_ptr, const char *pathname) {
    FILE *fp = fopen(pathname, "r");
    *lines_ptr = NULL;

    if (fp == NULL) {
        perror("Error opening file");
        exit(EXIT_FAILURE);
    }

    {
        int n = 0;
        char *line = NULL;
        size_t len = 0;
        char *line_copy = NULL;

        while ((n = getline(&line, &len, fp)) != -1) {
            if (line == NULL) {
                perror("Error reading line");
                exit(EXIT_FAILURE);
            }
            /* remove trailing newline */
            if (line[n - 1] == '\n') {
                line[n - 1] = '\0';
                n--;
            }
            line_copy = malloc(n);
            strncpy(line_copy, line, n);
            /*
             * the memory line_copy points to is now effectively "owned"
             * by the linked list, and it is freed by
             * free_str_linked_list
             */
            str_link_append(lines_ptr, line_copy);
        }
        fclose(fp);
        free(line);
    }

    if (*lines_ptr == NULL) {
        perror("Error reading lines into list");
        exit(EXIT_FAILURE);
    }
}

size_t parse_lines(int **arr_ptr, StrLink **lines_ptr) {
    /* returns number of lines parsed, -1 if list of lines is invalid */
    size_t n = str_linked_list_len(*lines_ptr);
    int i = 0;

    if (*lines_ptr == NULL) return -1;
    *arr_ptr = malloc(n * sizeof(int));

    for (StrLink *iter = *lines_ptr; iter != NULL; iter = iter->next) {
        (*arr_ptr)[i] = atoi(iter->data);
        i++;
    }
    return n;
}

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

int sum_reduce(const int arr[], size_t len) {
    int sum = 0;
    for (int i = 0; i < len; i++) {
        sum += arr[i];
    }
    return sum;
}

DECLARE_SUM_REDUCE_UNIVAR_INT_MAP_FUNC(total_fuel_required, fuel_required)

DECLARE_SUM_REDUCE_UNIVAR_INT_MAP_FUNC(total_fuel_required_recursive,
                                      fuel_required_recursive)

int main(void) {
    StrLink *lines = NULL;
    size_t num_modules;
    int *masses = NULL;

    readlines(&lines, "1.txt");
    num_modules = parse_lines(&masses, &lines);
    free_str_linked_list(&lines);

    printf("%d\n", total_fuel_required(masses, num_modules));
    printf("%d\n", total_fuel_required_recursive(masses, num_modules));
    free(masses);
    return(EXIT_SUCCESS);
}
