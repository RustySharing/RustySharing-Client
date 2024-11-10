import matplotlib.pyplot as plt
import re
import glob
import numpy as np

# List all files for both patterns: 'client_results_*.txt' and 'no_balancing_*.txt'
client_files = glob.glob('client_results_*.txt')
no_balancing_files = glob.glob('no_balancing_*.txt')

# Initialize lists to hold combined data for each type
client_runs = []
client_times = []
no_balancing_runs = []
no_balancing_times = []

# Process 'client_results_*.txt' files
for file_path in client_files:
    with open(file_path, 'r') as file:
        for line in file:
            match = re.search(r'Run: (\d+) - SUCCESS - .* - Time taken: ([\d.]+) seconds', line)
            if match:
                run = int(match.group(1))
                time_taken = float(match.group(2))
                
                # Append data to client-specific lists
                client_runs.append(run)
                client_times.append(time_taken)

# Process 'no_balancing_*.txt' files
for file_path in no_balancing_files:
    with open(file_path, 'r') as file:
        for line in file:
            match = re.search(r'Run: (\d+) - SUCCESS - .* - Time taken: ([\d.]+) seconds', line)
            if match:
                run = int(match.group(1))
                time_taken = float(match.group(2))
                
                # Append data to no-balancing-specific lists
                no_balancing_runs.append(run)
                no_balancing_times.append(time_taken)

# Calculate the line of best fit for 'client_results_*'
client_coeffs = np.polyfit(client_runs, client_times, 1)
client_poly = np.poly1d(client_coeffs)
client_fit_line = client_poly(np.array(client_runs))

# Calculate the line of best fit for 'no_balancing_*'
no_balancing_coeffs = np.polyfit(no_balancing_runs, no_balancing_times, 1)
no_balancing_poly = np.poly1d(no_balancing_coeffs)
no_balancing_fit_line = no_balancing_poly(np.array(no_balancing_runs))

# Plot both datasets with their respective best-fit lines on the same plot
plt.scatter(client_runs, client_times, label="Client Results Data", color='blue', alpha=0.5)
plt.plot(client_runs, client_fit_line, label="Client Results Best Fit", color='blue', linewidth=2)

plt.scatter(no_balancing_runs, no_balancing_times, label="No Balancing Data", color='green', alpha=0.5)
plt.plot(no_balancing_runs, no_balancing_fit_line, label="No Balancing Best Fit", color='green', linewidth=2)

# Customize plot
plt.title("Time Taken per Run: Client Results vs No Balancing with Line of Best Fit")
plt.xlabel("Run Number")
plt.ylabel("Time Taken (seconds)")
plt.legend()
plt.grid(True)

# Save the plot as an image
plt.savefig("combined_plot_with_best_fit.png")
