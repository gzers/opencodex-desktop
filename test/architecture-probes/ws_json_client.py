"""Small bounded WebSocket JSON transport for local protocol probes only."""
import base64,hashlib,json,os,socket,struct,time
class WsJson:
    def __init__(self,path=None,host='127.0.0.1',port=None,request_path='/',headers=None):
        if path:
            self.socket=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);self.socket.settimeout(5);self.socket.connect(str(path))
        else:self.socket=socket.create_connection((host,port),timeout=5)
        self.buffer=b'';self.closed=False
        key=base64.b64encode(os.urandom(16)).decode()
        extra=''.join(k+': '+v+'\r\n' for k,v in (headers or {}).items())
        self.socket.sendall(('GET '+request_path+' HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: '+key+'\r\nSec-WebSocket-Version: 13\r\n'+extra+'\r\n').encode())
        while b'\r\n\r\n' not in self.buffer:
            self.buffer+=self.socket.recv(4096)
            if len(self.buffer)>32768:raise RuntimeError('handshake too long')
        header,self.buffer=self.buffer.split(b'\r\n\r\n',1)
        if not header.startswith(b'HTTP/1.1 101'):raise RuntimeError('upgrade rejected: '+header.split(b'\r\n')[0].decode())
        accept=base64.b64encode(hashlib.sha1((key+'258EAFA5-E914-47DA-95CA-C5AB0DC85B11').encode()).digest())
        if accept.lower() not in header.lower():raise RuntimeError('invalid handshake accept')
    def send_frame(self,payload,opcode=1):
        mask=os.urandom(4);n=len(payload)
        prefix=bytes([0x80|opcode,0x80|(n if n<126 else 126 if n<65536 else 127)])
        if n>=126:prefix+=struct.pack('!H' if n<65536 else '!Q',n)
        self.socket.sendall(prefix+mask+bytes(v^mask[i%4] for i,v in enumerate(payload)))
    def send(self,message):self.send_frame(json.dumps(message).encode())
    def take(self,n):
        while len(self.buffer)<n:
            data=self.socket.recv(min(65536,n-len(self.buffer)))
            if not data:raise EOFError('WebSocket ended')
            self.buffer+=data
        value,self.buffer=self.buffer[:n],self.buffer[n:];return value
    def recv(self,deadline):
        fragments=b''
        while True:
            self.socket.settimeout(max(.01,deadline-time.monotonic()))
            a,b=self.take(2);opcode=a&15;n=b&127
            if n==126:n=struct.unpack('!H',self.take(2))[0]
            elif n==127:n=struct.unpack('!Q',self.take(8))[0]
            if n>8*1024*1024:raise RuntimeError('probe frame exceeds bound')
            mask=self.take(4) if b&128 else None;payload=self.take(n)
            if mask:payload=bytes(v^mask[i%4] for i,v in enumerate(payload))
            if opcode==8:raise EOFError('WebSocket closed')
            if opcode==9:self.send_frame(payload,10);continue
            if opcode in [0,1]:
                fragments+=payload
                if a&128:return json.loads(fragments)
    def close(self):
        if self.closed:return
        self.closed=True
        try:self.send_frame(b'',8)
        except OSError:pass
        self.socket.close()
