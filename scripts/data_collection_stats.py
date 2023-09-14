from collections import deque
import subprocess
from datetime import datetime

class DataCollectionMonitor:
    def __init__(self, log_file_path):
        self.log_file_path = log_file_path
        self.process = None
        self.speeds = deque(maxlen=60)
        self.curr_timestamp = None
        self.curr_lines = []
        self.buffer = ''

    def stop(self):
        self.process.kill()

    def run(self, on_speed_update: callable = None):
        self.process = subprocess.Popen(['tail', '-n 0', '-f', self.log_file_path], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        
        while True:
            if self.process.poll() is not None:
                break

            if self.curr_timestamp is None:
                chunk = self._initialize_timestamp()
            else:
                chunk = self._next_chunk()

            if chunk is None:
                break

            for timestamp, speed in self._process_chunk(chunk):
                if on_speed_update is not None:
                    on_speed_update(self, timestamp, speed)
    
    def get_average_speed(self, interval: int = 60) -> float:
        now = datetime.now()
        speeds_in_interval = []
        for timestamp, speed in reversed(self.speeds):
            if (diff := (now - timestamp).total_seconds()) <= interval:
                speeds_in_interval.append(speed)
            else:
                break
                
        if len(speeds_in_interval) == 0:
            return 0
        else:
            return sum(speeds_in_interval) / len(speeds_in_interval)
        

    def _next_chunk(self) -> list[str]:
        output = self.process.stdout.read(4096)
        if output == '' and self.process.poll() is not None:
            return None

        if output:
            new_lines_blob = self.buffer + output.decode('utf-8')
            new_lines = new_lines_blob.split('\n')
            self.buffer = new_lines.pop()
            return new_lines
        
    def _initialize_timestamp(self):
        # TODO: Implement max retries
        # If curr_datetime is None, then we are on the first line, so initialize curr_datetime to the first timestamp
        # If curr_datetime is not None, then we should not be calling this method
        # If we cannot find a timestamp in the chunk, then return False

        if self.curr_timestamp is not None:
            raise Exception('curr_timestamp is already initialized')
        
        while self.curr_timestamp is None:
            chunk = self._next_chunk()
            for line in chunk:
                try:
                    self.curr_timestamp = self._parse_timestamp(line)
                    return chunk
                
                except ValueError:
                    continue
    
    def _process_chunk(self, chunk):
        for line in chunk:
            try:
                timestamp = self._parse_timestamp(line)
            except ValueError:
                continue

            if timestamp == self.curr_timestamp:
                self.curr_lines.append(line)
            else:
                curr_speed = len(self.curr_lines)
                speed_update = (self.curr_timestamp, curr_speed)
                self.speeds.append(speed_update)

                self.curr_timestamp = timestamp
                self.curr_lines = [line]

                yield speed_update

    @staticmethod
    def _parse_timestamp(line):
        # TODO: Verify that the line is a price data point
        # Extract the timestamp from the line
        timestamp_str = line[1:20]

        # Convert the timestamp to a datetime object
        return datetime.strptime(timestamp_str, '%Y-%m-%d %H:%M:%S')

def on_speed_update(monitor, timestamp, speed):
    print(f'[{timestamp}] {speed} pts/sec | {monitor.get_average_speed():.2f} pts/sec (60s)')

def main():
    monitor = DataCollectionMonitor('logs/data-collection.log')
    monitor.run(on_speed_update)


if __name__ == '__main__':
    main()
    # l = [10, 56, 89, 62, 23, 39, 30, 46, 57, 42, 43, 51, 35, 47, 80, 61, 54, 34, 25, 60, 24, 89, 31, 50, 66, 40, 27, 69, 23, 25, 16, 15, 24, 29, 12, 24, 21, 85, 57, 40, 94, 36, 60, 50, 40, 16, 47, 37, 103, 55, 79, 71, 47, 69, 121, 104, 40, 25, 46]
    # print(len(l))