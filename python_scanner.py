import socket
import time
import argparse

parser = argparse.ArgumentParser()
parser.add_argument('--ip', required=True, help='Target IP address')
parser.add_argument('--ports', required=True, help='Port range, e.g., 1-1024')
parser.add_argument('--timeout', type=float, default=0.2)
args = parser.parse_args()

start_port, end_port = map(int, args.ports.split('-'))
ip = args.ip

print(f"Scanning {ip} ports {start_port}–{end_port} with timeout={args.timeout}s…")

start_time = time.time()

for port in range(start_port, end_port + 1):
    s = socket.socket()
    s.settimeout(args.timeout)
    try:
        s.connect((ip, port))
        print(f"Port {port:5} is OPEN")
    except:
        pass
    s.close()

print(f"\nScan complete in {time.time() - start_time:.2f} seconds.")
