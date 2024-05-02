import numpy as np
import matplotlib.pyplot as plt

REPORT_PATH = "backtest-results/generation_1.csv"

def main():
    data = np.loadtxt(REPORT_PATH, delimiter=',', skiprows=1)

    x = data[:, 0]
    balance = data[:, 1]
    position = data[:, 2]
    value = data[:, 3]
    signal = data[:, 4]
    bid = data[:, 5]
    ask = data[:, 6]
    fast_ma = data[:, 7]
    slow_ma = data[:, 8]

    value = value / value[0]

    # plt.plot(x, balance, label="balance")
    # plt.plot(x, position, label="position")
    plt.plot(x, value, label="value")
    plt.plot(x, bid, label="bid")
    plt.plot(x, fast_ma, label="fast_ma")
    plt.plot(x, slow_ma, label="slow_ma")

    # Plot a vertical red line where the signal is < 0 and a green line where the signal is > 0
    for i in range(len(signal)):
        if signal[i] < 0:
            plt.axvline(x=x[i], color='r', linestyle='--', linewidth=0.5)
        elif signal[i] > 0:
            plt.axvline(x=x[i], color='g', linestyle='--', linewidth=0.5)

    plt.legend()
    plt.show()

if __name__ == "__main__":
    main()
