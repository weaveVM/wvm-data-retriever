import requests
import time
import csv
import matplotlib.pyplot as plt

# Benchmark function to time HTTP GET requests
def benchmark_http_get(url, num_requests):
    response_times = []

    for i in range(num_requests):
        start_time = time.time()  # Start time before the request
        response = requests.get(url)
        end_time = time.time()  # End time after the request

        # Calculate the time taken for the request (in milliseconds)
        elapsed_time = (end_time - start_time) * 1000
        response_times.append(elapsed_time)

        print(f"Request {i + 1}: {elapsed_time:.2f} ms")
    
    return response_times

# Function to save benchmark results to CSV
def save_results_to_csv(response_times, file_name):
    with open(file_name, mode='w', newline='') as file:
        writer = csv.writer(file)
        writer.writerow(["Request Number", "Response Time (ms)"])
        for i, time in enumerate(response_times):
            writer.writerow([i + 1, time])

    print(f"Results saved to {file_name}")

# Function to plot the benchmark results with dark theme
def plot_results(response_times):
    plt.style.use('dark_background')  # Apply dark theme

    plt.plot(range(1, len(response_times) + 1), response_times, marker='o', color='#FF8C00')  # Dark orange color

    plt.title("wvm:// req for 1 KB calldata tx", color='white')
    plt.xlabel("Request Number", color='white')
    plt.ylabel("Response Time (ms)", color='white')

    plt.grid(True, linestyle='--', color='gray')
    plt.savefig("./media/1kb.png")
    plt.show()

# Main function to run the benchmark and plot the results
def main(txid):
    gateway = "https://gateway.wvm.dev"
    url = f"{gateway}/{txid}" 
    num_requests = 50  # Number of requests to perform

    # Run the benchmark and get response times
    response_times = benchmark_http_get(url, num_requests)

    # Plot the results
    plot_results(response_times)

if __name__ == "__main__":
    main("0xe14258b89ff9de351f046ada91e6aeb47842a6860dc1f79197e918334866eb87")
