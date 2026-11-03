from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import sys

class QaServer(ThreadingHTTPServer):
    request_queue_size = 256
    daemon_threads = True

server = QaServer(("127.0.0.1", 52429), partial(SimpleHTTPRequestHandler, directory=sys.argv[1]))
try:
    server.serve_forever()
except KeyboardInterrupt:
    pass
finally:
    server.server_close()
