import numpy as np
import matplotlib.pyplot as plt

REPORT_PATH = "../result.csv"

def main():
    data = np.loadtxt(REPORT_PATH, delimiter=',', skiprows=1)

    x = data[:, 0]
    balance = data[:, 1]
    position = data[:, 2]
    value = data[:, 3]

    # plt.plot(x, balance, label="balance")
    # plt.plot(x, position, label="position")
    plt.plot(x, value, label="value")
    plt.legend()
    plt.show()

if __name__ == "__main__":
    main()
