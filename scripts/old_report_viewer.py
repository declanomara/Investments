import sys
import matplotlib.pyplot as plt
import numpy as np

MAX_SIGNALS = 1000
def load_data(filename):
    data = np.genfromtxt(filename, delimiter=',', names=True)
    return data

def plot_signals(data):
    buy_mask = data['signal'] > 0
    sell_mask = data['signal'] < 0

    buy_timestamps = data[buy_mask]['timestamp']
    sell_timestamps = data[sell_mask]['timestamp']

    if (buy_count := len(buy_timestamps)) > MAX_SIGNALS:
        print(f"Too many buy signals to plot ({buy_count}/{len(data)}), skipping...")
        return

    if (sell_count := len(sell_timestamps)) > MAX_SIGNALS:
        print(f"Too many sell signals to plot ({sell_count}/{len(data)}), skipping...")
        return

    # Plot buy signals using axvline
    for ts in buy_timestamps:
        plt.axvline(x=ts, color='green', linestyle='--', linewidth=2)

    # Plot sell signals using axvline
    for ts in sell_timestamps:
        plt.axvline(x=ts, color='red', linestyle='--', linewidth=2)

def plot_data(data):
    plt.figure(figsize=(10, 6))  # Adjust the figure size if needed

    # Plot each column separately
    for column_name in data.dtype.names[1:]:  # Skip the first column (assuming it contains the x-axis data)
        # Skip the signal column, since we will plot it with vertical lines
        if column_name == 'signal':
            continue

        # Plot the 'value', 'cash' and 'position' columns on a secondary y-axis
        if column_name in ['value', 'cash', 'position']:
            continue

        if column_name == 'bid':
            plt.plot(data['timestamp'], data[column_name], label=column_name, color='grey', alpha=0.5)
            continue

        if column_name == 'ask':
            plt.plot(data['timestamp'], data[column_name], label=column_name, color='lightgrey', alpha=0.5)
            continue

        if column_name == 'slow_ma':
            plt.plot(data['timestamp'], data[column_name], label=column_name, color='royalblue')
            continue

        if column_name == 'fast_ma':
            plt.plot(data['timestamp'], data[column_name], label=column_name, color='orange')
            continue

        plt.plot(data['timestamp'], data[column_name], label=column_name)

    # Plot the signals
    plot_signals(data)
    print(f"Total number of signals: {len(data[data['signal'] != 0])}")

    plt.xlabel('Time')  # Replace with your actual X-axis label
    plt.ylabel('Price')  # Replace with your actual Y-axis label
    plt.title('Trends in currency pair')  # Replace with your desired title
    plt.legend()  # Show the legend using the labels specified in the plot() function

    plt.grid(True)  # Add grid lines to the plot if desired
    plt.show()  # Display the plot

if __name__ == '__main__':
    filename = sys.argv[1]
    data = load_data(filename)
    plot_data(data)
