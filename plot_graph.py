import matplotlib.pyplot as plt
import re
import glob
import numpy as np

# List all files with a certain pattern, e.g., 'run_data_*.txt'
file_paths = glob.glob('client_results_*.txt')

# Initialize lists to hold combined data
all_runs = []
all_times = []

# Loop over each file to extract data
for file_path in file_paths:
    with open(file_path, 'r') as file:
        for line in file:
            match = re.search(r'Run: (\d+) - SUCCESS - .* - Time taken: ([\d.]+) seconds', line)
            if match:
                run = int(match.group(1))
                time_taken = float(match.group(2))
                
                # Append data to combined lists
                all_runs.append(run)
                all_times.append(time_taken)

# Calculate the line of best fit using all combined data
coeffs = np.polyfit(all_runs, all_times, 1)  # Fit a line of degree 1 (linear)
poly = np.poly1d(coeffs)

# Generate the values for the line of best fit
fit_line = poly(np.array(all_runs))

# Plot the combined data points and the line of best fit
plt.scatter(all_runs, all_times, label="Data Points", color='blue', alpha=0.5)  # Scatter plot for all data points
plt.plot(all_runs, fit_line, label="Best Fit Line", color='red', linewidth=2)  # Line of best fit

# Customize plot
plt.title("Combined Time Taken per Run with Line of Best Fit")
plt.xlabel("Run Number")
plt.ylabel("Time Taken (seconds)")
plt.legend()
plt.grid(True)

# Save the plot as an image
plt.savefig("combined_plot_with_best_fit.png")
