#!/usr/bin/env python3

# This script is ran at the end of the week to move the current week's data to the archive folder.

# The structure of the archive folder is as follows:
# archive/
#    weekly/
#        {instrument}/
#            {year}-{week}.bin
#    raw/
#        {year}-{week}.log

# The structure of the data folder is as follows:
# data/
#    raw.log
#    bin/
#        {instrument}.bin

# Script usage: python3 end_of_week.py [-v] [-h] [-w] [-y] {data_dir} {archive_dir}

import shutil
import argparse
import datetime
from pathlib import Path

def archive_raw_data(data_dir: Path, archive_dir: Path, week: int, year: int, verbose: bool = False) -> None:
    # Archive the raw data
    raw_dir = archive_dir / 'raw'
    raw_file = Path(data_dir) / 'raw.log'
    raw_archive_file = raw_dir / f'{year}-{week}.log'
    if raw_file.exists():
        shutil.move(raw_file, raw_archive_file)
        print(f'Archived raw data to {raw_archive_file}')


def archive_binary_data(data_dir: Path, archive_dir: Path, week: int, year: int, verbose: bool = False, force: bool = False) -> None:
    # Archive the binary data
    archived_bin_dir = archive_dir / 'weekly'
    print(f'Archiving binary data to {archived_bin_dir}')

    bin_dir = Path(data_dir) / 'bin'
    for bin_file in bin_dir.iterdir():
        # Get the instrument name from the file name
        instrument = bin_file.stem
        archived_instrument_dir = archived_bin_dir / instrument
        archived_bin_file = archived_instrument_dir / f'{year}-{week}.bin'

        # Create the instrument directory if it doesn't exist
        archived_instrument_dir.mkdir(parents=True, exist_ok=True)
        
        if archived_bin_file.exists():
            if force:
                print(f'Overwriting archived binary data for {instrument}')
            else:
                print(f'Archived binary data for {instrument} already exists, rerun with -f to overwrite')
                continue

        # Move the binary file
        shutil.move(bin_file, archived_bin_file)


def ensure_archive_dir(archive_dir: Path) -> None:
    # Create the archive directory if it doesn't exist
    archive_dir.mkdir(parents=True, exist_ok=True)

    # Create the weekly directory if it doesn't exist
    weekly_dir = archive_dir / 'weekly'
    weekly_dir.mkdir(exist_ok=True)

    # Create the raw directory if it doesn't exist
    raw_dir = archive_dir / 'raw'
    raw_dir.mkdir(exist_ok=True)


def archive_data(data_dir: str, archive_dir: str, verbose: bool, week: int = None, year: int = None, force: bool = False) -> None:
    calculated_year, calculated_week, _ = datetime.date.today().isocalendar()
    if week is None:
        week = calculated_week
    if year is None:
        year = calculated_year

    print(f'Archiving data for week {week} of {year}')

    # Ensure the archive directory exists
    archive_dir = Path(archive_dir)
    ensure_archive_dir(archive_dir)

    # Archive the raw data
    archive_raw_data(data_dir, archive_dir, week, year, verbose)

    # Archive the binary data
    archive_binary_data(data_dir, archive_dir, week, year, verbose, force)
    

def main() -> None:
    parser = argparse.ArgumentParser(description='Description of your program')

    parser.add_argument('data_dir', type=str, help='Path to the data folder')
    parser.add_argument('archive_dir', type=str, help='Path to the archive folder')
    parser.add_argument('-v', '--verbose', action='store_true', help='Enable verbose output')
    parser.add_argument('-w', '--week', type=int, help='Week number to archive')
    parser.add_argument('-y', '--year', type=int, help='Year number to archive')
    parser.add_argument('-f', '--force', action='store_true', help='Force overwrite of existing files')

    try:
        args = parser.parse_args()
    except argparse.ArgumentError as e:
        print(str(e))
        parser.print_usage()
        sys.exit(1)

    archive_data(args.data_dir, args.archive_dir, args.verbose, week=args.week, year=args.year, force=args.force)


if __name__ == '__main__':
    main()