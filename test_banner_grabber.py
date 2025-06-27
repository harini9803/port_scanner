#!/usr/bin/env python3
"""
Test script to demonstrate banner grabbing functionality
This script creates simple servers for HTTP, FTP, and SMTP to test banner grabbing
"""

import socket
import threading
import time
import sys

def http_server(port=8080):
    """Simple HTTP server for banner grabbing test"""
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(('localhost', port))
    server.listen(1)
    
    print(f"HTTP server listening on port {port}")
    
    while True:
        try:
            client, addr = server.accept()
            print(f"HTTP: Connection from {addr}")
            
            response = """HTTP/1.1 200 OK
Server: TestServer/1.0
X-Powered-By: Python
Content-Type: text/html
Content-Length: 25

<h1>Test HTTP Server</h1>"""
            
            client.send(response.encode())
            client.close()
            
        except KeyboardInterrupt:
            break
    
    server.close()

def ftp_server(port=2121):
    """Simple FTP server for banner grabbing test"""
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(('localhost', port))
    server.listen(1)
    
    print(f"FTP server listening on port {port}")
    
    while True:
        try:
            client, addr = server.accept()
            print(f"FTP: Connection from {addr}")
            
            banner = "220 Test FTP Server (vsFTPd 3.0.3) ready.\r\n"
            client.send(banner.encode())
            
            data = client.recv(1024)
            if data:
                if b"HELP" in data:
                    help_response = """214-The following commands are recognized:
214-ABOR ACCT ALLO APPE CDUP CWD  DELE EPRT EPSV FEAT HELP LIST MDTM MKD
214-MODE NLST NOOP OPTS PASS PASV PORT PWD  QUIT REIN REST RETR RMD  RNFR
214-RNTO SITE SIZE SMNT STAT STOR STOU STRU SYST TYPE USER XCUP XCWD XMKD
214-XPWD XRMD
214 Help OK.\r\n"""
                    client.send(help_response.encode())
            
            client.close()
            
        except KeyboardInterrupt:
            break
    
    server.close()

def smtp_server(port=2525):
    """Simple SMTP server for banner grabbing test"""
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(('localhost', port))
    server.listen(1)
    
    print(f"SMTP server listening on port {port}")
    
    while True:
        try:
            client, addr = server.accept()
            print(f"SMTP: Connection from {addr}")
            
            banner = "220 test.local ESMTP Postfix (Ubuntu)\r\n"
            client.send(banner.encode())
            
            data = client.recv(1024)
            if data and b"EHLO" in data:
                ehlo_response = """250-test.local
250-PIPELINING
250-SIZE 10240000
250-VRFY
250-ETRN
250-STARTTLS
250-ENHANCEDSTATUSCODES
250-8BITMIME
250 DSN\r\n"""
                client.send(ehlo_response.encode())
            
            client.close()
            
        except KeyboardInterrupt:
            break
    
    server.close()

def main():
    if len(sys.argv) < 2:
        print("Usage: python test_banner_grabber.py [http|ftp|smtp|all]")
        sys.exit(1)
    
    protocol = sys.argv[1].lower()
    
    if protocol == "http":
        http_server()
    elif protocol == "ftp":
        ftp_server()
    elif protocol == "smtp":
        smtp_server()
    elif protocol == "all":
        threads = []
        
        http_thread = threading.Thread(target=http_server, args=(8080,))
        ftp_thread = threading.Thread(target=ftp_server, args=(2121,))
        smtp_thread = threading.Thread(target=smtp_server, args=(2525,))
        
        threads.extend([http_thread, ftp_thread, smtp_thread])
        
        for thread in threads:
            thread.daemon = True
            thread.start()
        
        print("All test servers started:")
        print("- HTTP server on port 8080")
        print("- FTP server on port 2121")
        print("- SMTP server on port 2525")
        print("Press Ctrl+C to stop all servers")
        
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            print("\nShutting down servers...")
    else:
        print("Invalid protocol. Use: http, ftp, smtp, or all")

if __name__ == "__main__":
    main() 