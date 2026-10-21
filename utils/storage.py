import struct

msb='little'

class Reader:
    def __init__(self, fs, metastr):
        self.fs = fs
        self.metastr = metastr
        self.actors = {}
        self.worlds = {}

    def read_u(self, size):
        return int.from_bytes(self.fs.read(size), msb)

    def u8(self):
        return self.read_u(1)

    def u16(self):
        return self.read_u(2)

    def u32(self):
        return self.read_u(4)

    def u64(self):
        return self.read_u(8)

    def f32(self):
        return struct.unpack('<f', self.fs.read(4))[0]

    def bool(self):
        return (self.u8() != 0)

    def utf8(self):
        return self.fs.read(self.u16()).decode('utf-8')

    def array(self, alist, reader):
        acount = self.u32()
        alist.clear()
        for i in range(acount):
            alist.append( reader(self) )

class Writer:
    def __init__(self, fs):
        self.fs = fs

    def write_u(self, value, size):
        self.fs.write( value.to_bytes(size, msb) )

    def u8(self, value):
        return self.write_u(value, 1)

    def u16(self, value):
        return self.write_u(value, 2)

    def u32(self, value):
        return self.write_u(value, 4)

    def u64(self, value):
        return self.write_u(value, 8)

    def f32(self, value):
        return self.fs.write( struct.pack('<f', value) )

    def bool(self, value):
        if value:
            self.u8(1)
        else:
            self.u8(0)

    def utf8(self, value):
        data = value.encode('utf-8')
        self.u16(len(data))
        self.fs.write(data)

    def array(self, alist, writer):
        self.u32(len(alist))
        for item in alist:
            writer(item, self)

class Depot:
    def __init__(self):
        self.data = []

    @classmethod
    def load(cls, reader, itemreader):
        depot = cls()
        for i in range(reader.u32()):
            if reader.bool():
                depot.data.append( itemreader(reader) )
            else:
                depot.data.append( None )
        return depot

    def save(self, writer, itemwriter):
        writer.u32(len(self.data))
        for item in self.data:
            if item == None:
                writer.bool(False)
            else:
                writer.bool(True)
                itemwriter(item, writer)


