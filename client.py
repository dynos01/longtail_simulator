import datetime
import socket
import sys
from threading import Thread

class Config:
    def __init__(self, dst, n, c):
        self.dst = dst
        self.n = n
        self.c = c

def parse_config():
    try:
        (ip, port) = sys.argv[1].split(':')

        _ = socket.inet_aton(ip)
        port = int(port)

        n = int(sys.argv[2])
        c = int(sys.argv[3])

        if port <= 0 or port > 65535 or n < 0 or c < 0:
            return None

        return Config((ip, port), n, c)
    except:
        return None

def request(dst):
    try:
        start = datetime.datetime.now()

        client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        client.connect(dst)
        client.send(b'GET / HTTP/1.1\r\n\r\n')

        response = ''
        while True:
            recv = client.recv(1024).decode()
            if not recv:
                break
            response += recv

        if response.startswith('HTTP/1.1 200 OK'):
            end = datetime.datetime.now()
            print((end - start).total_seconds() * 1000)

        #print(end, start)

        client.close()
    except:
        print(-1)

def main():
    config = parse_config()

    if config is None:
        print('Usage: python client.py DESTINATION COUNT N_CONCURRENT', file=sys.stderr)
        sys.exit(1)

    while config.n > 0:
        count = min(config.c, config.n)

        handles = []

        for i in range(count):
            handles.append(Thread(target=request, args=[config.dst]))
            handles[-1].start()

        for i in handles:
            i.join()

        config.n -= count

if __name__ == '__main__':
    main()