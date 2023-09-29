data = {
    0: 1.594283e-3,
    1: 963.239521e-3,
    2: 1.339846082,
    3: 1.434744464,
    4: 1.418355682,
    5: 1.441986651,
    6: 1.483104784,
    7: 1.42036839,
    8: 1.42964112,
    9: 1.475931994,
    10: 1.449848983,
    11: 1.601475148,
    12: 1.581926081,
    13: 1.805979212,
    14: 1.6244297,
    15: 1.661819956,
    16: 2.002322952,
    17: 3.088042119,
    18: 2.54949503,
    19: 4.586132389,
}

# Calculate throughput for each batch size and find the batch size with the highest throughput
throughput_data = {batch_size: batch_size / search_time for batch_size, search_time in data.items()}
max_throughput_batch_size = max(throughput_data, key=throughput_data.get)
max_throughput_value = throughput_data[max_throughput_batch_size]

print(max_throughput_batch_size, max_throughput_value)