#include <stdio.h>

void selectionSort(int arr[], int n) {
    int i, j, min_idx;

    // One by one move boundary of unsorted subarray
    for (i = 0; i < n; i++) {
        // Find the minimum element in unsorted array
        min_idx = i;
        for (j = i; j < n; j++)
            if (arr[j] < arr[min_idx])
                min_idx = j;

        // Swap the found minimum element with the first element
        if (min_idx != i) {
            int t = arr[min_idx];
            arr[min_idx] = arr[i];
            arr[i] = t;
        }

    }
}


int main() {
    int data[] = {5, 4, 3, 2, 1};
    selectionSort(data, 5);

    for (int i = 0; i < 5; ++i) {
        printf("%i ", data[i]);
    }
    printf("\n");
    return 0;
}