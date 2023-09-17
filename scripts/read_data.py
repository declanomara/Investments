# !/usr/bin/env python3

# This script is a utility script to read the binary data files and print them to stdout.
# Usage: python3 read_data.py [-n {num_rows}] [-h] binary_file

import argparse
import datetime
import struct

from pathlib import Path

class Row:
    def __init__(self, timestamp: int, bid: float, ask: float):
        try:
            self.timestamp = datetime.datetime.fromtimestamp(timestamp / 1000) # Convert from milliseconds to seconds and then to datetime
        except ValueError:
            raise ValueError(f'Invalid timestamp: {timestamp} (is the byte order correct?)')

        self.bid = bid
        self.ask = ask
    
    def __str__(self):
        return f'{self.timestamp} Bid: {self.bid} Ask: {self.ask}'
    
    def __repr__(self):
        return str(self)
    
    @staticmethod
    def from_bytes(data: bytes, byte_order: str = 'little'):
        byte_order = '>' if byte_order == 'big' else '<'
        timestamp, bid, ask = struct.unpack(byte_order + 'Qff', data)
        return Row(timestamp, bid, ask)
    
    def to_bytes(self):
        return struct.pack('Qff', int(self.timestamp.timestamp()), self.bid, self.ask)
    

def read_binary_file(binary_file: Path, byte_order: str = 'little', num_rows: int = None) -> None:
    if num_rows is None:
        num_rows = int(1e9)

    with open(binary_file, 'rb') as f:
        for i in range(num_rows):
            data = f.read(16)
            if len(data) < 16:
                print(f'End of file reached after {i} rows')
                break
            row = Row.from_bytes(data, byte_order=byte_order)
            yield row

def main():
    parser = argparse.ArgumentParser(description='Read binary data file')
    parser.add_argument('binary_file', type=str, help='Binary data file to read')
    parser.add_argument('-n', '--num_rows', type=int, help='Number of rows to read')
    parser.add_argument('-b', '--byte_order', type=str, default='little', help='Byte order of the binary file')
    args = parser.parse_args()

    binary_file = Path(args.binary_file)
    num_rows = args.num_rows
    byte_order = args.byte_order

    for row in read_binary_file(binary_file, byte_order=byte_order ,num_rows=num_rows):
        print(row)

if __name__ == '__main__':
    main()