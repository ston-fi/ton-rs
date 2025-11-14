#!/usr/bin/env python3
"""
Benchmark runner for tx_emulator_bench.

Runs cargo bench with specified parameters and parses results into a table.
"""

import argparse
import csv
import subprocess
import re
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict
from tabulate import tabulate


def parse_benchmark_output(output: str) -> Dict[str, float]:
    """
    Parse cargo bench output to extract benchmark results.
    
    Returns a dictionary mapping benchmark names to time in microseconds.
    Handles two formats:
    1. Single line: benchmark_name        time:   [X.XX ms Y.YY ms Z.ZZ ms]
    2. Multi-line: benchmark_name (on one line), time: [X.XX ms Y.YY ms Z.ZZ ms] (on next line)
    We extract the middle value (Y.YY ms) and convert to microseconds.
    Handles ANSI color codes in the output.
    """
    results = {}
    
    # Remove ANSI escape codes for cleaner parsing
    ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
    clean_output = ansi_escape.sub('', output)
    
    lines = clean_output.split('\n')
    
    # Pattern to match: benchmark_name        time:   [X.XX ms Y.YY ms Z.ZZ ms] (single line)
    single_line_pattern = r'(\S+)\s+time:\s+\[[\d.]+\s+ms\s+([\d.]+)\s+ms\s+[\d.]+\s+ms\]'
    
    # Pattern to match: time:   [X.XX ms Y.YY ms Z.ZZ ms] (multi-line, benchmark name on previous line)
    time_line_pattern = r'^\s+time:\s+\[[\d.]+\s+ms\s+([\d.]+)\s+ms\s+[\d.]+\s+ms\]'
    
    i = 0
    while i < len(lines):
        line = lines[i]
        
        # Try single-line format first
        match = re.search(single_line_pattern, line)
        if match:
            benchmark_name = match.group(1)
            time_ms = float(match.group(2))
            time_us = time_ms * 1000  # Convert milliseconds to microseconds
            results[benchmark_name] = time_us
            i += 1
            continue
        
        # Try multi-line format: benchmark name on current line, time on next line
        if i + 1 < len(lines):
            next_line = lines[i + 1]
            time_match = re.search(time_line_pattern, next_line)
            if time_match:
                # Current line should be just the benchmark name (possibly with whitespace)
                benchmark_name = line.strip()
                if benchmark_name and not benchmark_name.startswith('time:'):
                    time_ms = float(time_match.group(1))
                    time_us = time_ms * 1000  # Convert milliseconds to microseconds
                    results[benchmark_name] = time_us
                    i += 2  # Skip both lines
                    continue
        
        i += 1
    
    return results


def run_benchmark(
    path: Path,
    mode: int,
    threads: int,
    pin_to_core: bool,
    test_dir: Path
) -> str:
    """
    Run cargo bench with specified parameters.
    STDERR is redirected to /dev/null, only stdout is captured.
    
    Returns: stdout
    """
    cmd = [
        'cargo', 'bench',
        '--bench', 'tx_emulator_bench',
        '--features', 'tonlibjson',
        '--',
        '--pin-to-core', 'true' if pin_to_core else 'false',
        '--mode', str(mode),
        '--threads', str(threads),
    ]
    
    # Run the benchmark from the ton-rs directory
    # Redirect STDERR to /dev/null, only capture stdout
    process = subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        timeout=300,  # 5 minute timeout
        cwd=str(path)  # Run from the ton-rs directory
    )
    
    stdout = process.stdout or ''
    
    # Save output to file in the test directory
    output_file = test_dir / f'bench_mode{mode}_threads{threads}.txt'
    with open(output_file, 'w') as f:
        f.write('=== COMMAND ===\n')
        f.write(' '.join(cmd) + '\n')
        f.write('\n=== STDOUT ===\n')
        f.write(stdout)
        f.write(f'\n=== RETURN CODE ===\n{process.returncode}\n')
    
    if process.returncode != 0:
        print(f"Warning: Benchmark failed with return code {process.returncode}", file=sys.stderr)
        print(f"Output saved to: {output_file}", file=sys.stderr)
    
    return stdout


def main():
    parser = argparse.ArgumentParser(
        description='Run tx_emulator_bench and parse results into a table'
    )
    parser.add_argument(
        '--path',
        type=str,
        required=True,
        help='Path to ton-rs folder'
    )
    parser.add_argument(
        '--work_dir',
        type=str,
        required=True,
        help='Directory to store output files'
    )
    parser.add_argument(
        '--pin-to-core',
        type=str,
        choices=['true', 'false'],
        default='false',
        help='Pin threads to specific cores (true/false)'
    )
    parser.add_argument(
        '--mode',
        type=int,
        required=True,
        choices=[0, 1, 2, 3, 4, 5],
        help='Mode integer: 0 = run all modes (1,2,3,4,5), 1-5 = specific mode (1: SleepTest, 2: EmulatorTest, 3: RecreateEmulTest, 4: AutoPoolAsyncEmul, 5: CpuLoadTest)'
    )
    parser.add_argument(
        '--threads',
        type=str,
        required=True,
        help='Comma-separated list of thread counts (e.g., "1,3,4")'
    )
    
    args = parser.parse_args()
    
    # Validate and parse paths
    path = Path(args.path).resolve()
    if not path.exists():
        print(f"Error: Path does not exist: {path}", file=sys.stderr)
        sys.exit(1)
    
    work_dir = Path(args.work_dir).resolve()
    work_dir.mkdir(parents=True, exist_ok=True)
    
    # Convert pin-to-core string to boolean
    pin_to_core_bool = args.pin_to_core.lower() == 'true'
    
    # Determine which modes to run
    if args.mode == 0:
        modes_to_run = [1, 2, 3, 4, 5]
    else:
        modes_to_run = [args.mode]
    
    # Parse thread counts
    try:
        thread_counts = [int(t.strip()) for t in args.threads.split(',')]
    except ValueError:
        print(f"Error: Invalid thread format: {args.threads}. Expected comma-separated integers.", file=sys.stderr)
        sys.exit(1)
    
    if not thread_counts:
        print("Error: At least one thread count must be specified", file=sys.stderr)
        sys.exit(1)
    
    # Create a unique test folder for this run
    # Format: {mode}_isol{true/false}_{HHMMSS} or 0_isol{true/false}_{HHMMSS} for all modes
    time_str = datetime.now().strftime('%H%M%S')  # Hours, minutes, seconds
    test_dir_name = f'{args.mode}_isol{args.pin_to_core}_{time_str}'
    test_dir = work_dir / test_dir_name
    test_dir.mkdir(parents=True, exist_ok=True)
    
    # Save test configuration
    full_timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    config_file = test_dir / 'config.txt'
    with open(config_file, 'w') as f:
        f.write(f"Test Configuration\n")
        f.write(f"{'=' * 50}\n")
        f.write(f"Timestamp: {full_timestamp}\n")
        f.write(f"Mode: {args.mode} {'(all modes: 1,2,3,4,5 - SleepTest, EmulatorTest, RecreateEmulTest, AutoPoolAsyncEmul, CpuLoadTest)' if args.mode == 0 else ''}\n")
        f.write(f"Pin-to-core: {args.pin_to_core}\n")
        f.write(f"Threads: {args.threads}\n")
        f.write(f"Path: {path}\n")
        f.write(f"Work Dir: {work_dir}\n")
        f.write(f"Test Dir: {test_dir}\n")
    
    # Run benchmarks for each mode and thread count
    # Structure: all_results[mode][threads][benchmark_name] = time_us
    all_results: Dict[int, Dict[int, Dict[str, float]]] = {}
    
    print(f"Running benchmarks with mode={args.mode} {'(all modes)' if args.mode == 0 else ''}, threads={thread_counts}, pin-to-core={args.pin_to_core}")
    print(f"Test directory: {test_dir}\n")
    
    for mode in modes_to_run:
        all_results[mode] = {}
        print(f"\n{'='*60}")
        print(f"Running benchmarks for MODE {mode}")
        print(f"{'='*60}\n")
        
        for threads in thread_counts:
            print(f"Running benchmark with mode={mode}, threads={threads}...")
            stdout = run_benchmark(path, mode, threads, pin_to_core_bool, test_dir)
            
            # Parse results
            results = parse_benchmark_output(stdout)
            if results:
                all_results[mode][threads] = results
                print(f"  Found {len(results)} benchmark(s)")
            else:
                print(f"  Warning: No benchmark results found in output")
                all_results[mode][threads] = {}
    
    # Create table
    if not all_results:
        print("\nError: No benchmark results were found. Check the output files.", file=sys.stderr)
        sys.exit(1)
    
    # Get all unique benchmark names across all modes
    all_benchmarks = set()
    for mode_results in all_results.values():
        for thread_results in mode_results.values():
            all_benchmarks.update(thread_results.keys())
    all_benchmarks = sorted(all_benchmarks)
    
    if not all_benchmarks:
        print("\nError: No benchmark names found.", file=sys.stderr)
        sys.exit(1)
    
    # Build table data
    table_data = []
    
    if args.mode == 0:
        # For mode 0, map each benchmark to its mode and create one row per thread
        # Find which mode has which benchmark
        benchmark_to_mode: Dict[str, int] = {}
        for mode in sorted(modes_to_run):
            if mode in all_results:
                for threads in all_results[mode]:
                    for benchmark in all_results[mode][threads]:
                        if benchmark not in benchmark_to_mode:
                            benchmark_to_mode[benchmark] = mode
        
        # Sort benchmarks alphabetically to ensure consistent column order
        # Expected benchmarks: autopool_async_emul_bench, cpu_task_bench, emulator_task_bench, emulator_task_bench_recreate, sleep_task_bench
        # Mode 1: sleep_task_bench, Mode 2: emulator_task_bench, Mode 3: emulator_task_bench_recreate, Mode 4: autopool_async_emul_bench, Mode 5: cpu_task_bench
        sorted_benchmarks = sorted(all_benchmarks)
        
        # Create table with one row per thread
        headers = ['Threads'] + sorted_benchmarks
        
        for threads in sorted(thread_counts):
            row = [str(threads)]
            for benchmark in sorted_benchmarks:
                # Find the mode that has this benchmark
                if benchmark in benchmark_to_mode:
                    mode = benchmark_to_mode[benchmark]
                    if mode in all_results and threads in all_results[mode] and benchmark in all_results[mode][threads]:
                        time_us = all_results[mode][threads][benchmark]
                        row.append(f"{time_us:.2f} μs")
                    else:
                        row.append("N/A")
                else:
                    row.append("N/A")
            table_data.append(row)
    else:
        # For single mode, show: Threads, Benchmark1, Benchmark2, ...
        headers = ['Threads'] + sorted(all_benchmarks)
        
        for threads in sorted(thread_counts):
            if args.mode in all_results and threads in all_results[args.mode]:
                row = [str(threads)]
                for benchmark in sorted(all_benchmarks):
                    if benchmark in all_results[args.mode][threads]:
                        time_us = all_results[args.mode][threads][benchmark]
                        row.append(f"{time_us:.2f} μs")
                    else:
                        row.append("N/A")
                table_data.append(row)
    
    # Print table
    print("\n" + "=" * 80)
    print("Benchmark Results (time in microseconds)")
    print("=" * 80)
    print(tabulate(table_data, headers=headers, tablefmt='grid'))
    print()
    
    # Save table to file in the test directory
    full_timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    table_file = test_dir / f'results_mode{args.mode}.txt'
    with open(table_file, 'w') as f:
        f.write("Benchmark Results (time in microseconds)\n")
        f.write("=" * 80 + "\n")
        f.write(f"Mode: {args.mode} {'(all modes: 1,2,3,4,5 - SleepTest, EmulatorTest, RecreateEmulTest, AutoPoolAsyncEmul, CpuLoadTest)' if args.mode == 0 else ''}\n")
        f.write(f"Pin-to-core: {args.pin_to_core}\n")
        f.write(f"Threads: {args.threads}\n")
        f.write(f"Timestamp: {full_timestamp}\n")
        f.write("=" * 80 + "\n\n")
        f.write(tabulate(table_data, headers=headers, tablefmt='grid'))
        f.write("\n")
    
    # Save table as CSV
    csv_file = test_dir / f'results_mode{args.mode}.csv'
    with open(csv_file, 'w', newline='') as f:
        writer = csv.writer(f)
        # Write headers
        writer.writerow(headers)
        # Write data rows
        for row in table_data:
            writer.writerow(row)
    
    print(f"\nAll results saved to test directory: {test_dir}")
    print(f"  - Configuration: {config_file.name}")
    print(f"  - Summary table (text): {table_file.name}")
    print(f"  - Summary table (CSV): {csv_file.name}")
    if args.mode == 0:
        print(f"  - Individual outputs: bench_mode1_threads*.txt, bench_mode2_threads*.txt, bench_mode3_threads*.txt, bench_mode4_threads*.txt, bench_mode5_threads*.txt")
        print(f"    (Mode 1: SleepTest, Mode 2: EmulatorTest, Mode 3: RecreateEmulTest, Mode 4: AutoPoolAsyncEmul, Mode 5: CpuLoadTest)")
    else:
        print(f"  - Individual outputs: bench_mode{args.mode}_threads*.txt")


if __name__ == '__main__':
    main()

