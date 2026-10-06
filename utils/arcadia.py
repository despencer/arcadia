import os
import struct
import arhistory
import units

msb='little'
UNIVERSE_VERSION=1

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

class BluePrint:
    FVALUE = 1
    UVALUE = 2
    INSTRUCTION = 3
    COMPOUND = 4
    ARCHITECTURE = 5
    REFERENCE = 6

    def __init__(self):
        self.btype = 0
        self.unit = 0
        self.value = None
        self.components = None

    @classmethod
    def make_from(cls, metastr, jvalue):
        bp = cls()
        bp.btype = cls.ARCHITECTURE
        bp.components = []
        bp.value = cls()
        bp.value.btype = cls.COMPOUND
        bp.value.unit = ''
        bp.value.value = []
        for jv in jvalue:
            bp.value.value.append( cls.make_unit(metastr, jv) )
        return bp

    @classmethod
    def make_unit(cls, metastr, jvalue):
        bp = cls()
        if jvalue[0][0] == '$':
            bp.btype = cls.INSTRUCTION
            bp.unit = metastr.get_unit(jvalue[0][1:]).id
            bp.value = []
            for jv in jvalue[1:]:
                bp.value.append( cls.make_value(jv) )
        else:
            bp.btype = cls.COMPOUND
            bp.unit = jvalue[0]
            bp.value = []
            for jv in jvalue[1:]:
                bp.value.append( cls.make_unit(metastr, jv) )
        return bp

    @classmethod
    def make_value(cls, jvalue):
        bp = cls()
        if isinstance(jvalue, int):
            bp.btype = cls.UVALUE
            bp.value = jvalue
        elif isinstance(jvalue, float):
            bp.btype = cls.FVALUE
            bp.value = jvalue
        elif isinstance(jvalue, str):
            bp.btype = cls.REFERENCE
        else:
            raise Exception(f'Bad blueprint value {jvalue}')
        return bp

    @classmethod
    def load(cls, reader):
        bp = cls()
        bp.btype = reader.u8()
        if bp.btype == cls.FVALUE:
            bp.value = reader.f32()
        elif bp.btype == cls.UVALUE:
            bp.value = reader.u32()
        elif bp.btype == cls.INSTRUCTION:
            bp.unit = reader.u16()
            bp.value = []
            reader.array(bp.value, cls.load)
        elif bp.btype == cls.COMPOUND:
            bp.unit = reader.utf8()
            bp.value = []
            reader.array(bp.value, cls.load)
        elif bp.btype == cls.ARCHITECTURE:
            bp.components = []
            reader.array(bp.components, cls.load)
            bp.value = cls.load(reader)
        elif bp.btype == cls.REFERENCE:
            pass
        else:
            raise Exception(f'Unknown blueprint type {bp.btype}')
        return bp

    def save(self, writer):
        writer.u8(self.btype)
        if self.btype == self.FVALUE:
            writer.f32(self.value)
        elif self.btype == self.UVALUE:
            writer.u32(self.value)
        elif self.btype == self.INSTRUCTION:
            writer.u16(self.unit)
            writer.array(self.value, BluePrint.save)
        elif self.btype == self.COMPOUND:
            writer.utf8(self.unit)
            writer.array(self.value, BluePrint.save)
        elif self.btype == self.ARCHITECTURE:
            writer.array(self.components, BluePrint.save)
            self.value.save(writer)
        elif self.btype == self.REFERENCE:
            pass
        else:
            raise Exception(f'Unknown blueprint type {self.bptype}')

    def __repr__(self):
        if self.btype == self.FVALUE:
            return f'{self.value:.1f}'
        elif self.btype == self.UVALUE:
            return f'{self.value}'
        elif self.btype == self.INSTRUCTION:
            return "["+ f'${self.unit}: ' + ', '.join(map(str, self.value)) +"]"
        elif self.btype == self.COMPOUND:
            return "["+ f'"{self.unit}": ' + ', '.join(map(str, self.value)) +"]"
        elif self.btype == self.ARCHITECTURE:
            return "<["+ ', '.join(map(str, self.components)) + f"] | {self.value}>"
        elif self.btype == self.REFERENCE:
            return '#'
        raise Exception(f'Unknown blueprint type {self.bptype}')

class Seed:
    def __init__(self):
        self.credits = 0
        self.seed = Compartment()

    @classmethod
    def load(cls, reader):
        seed = cls()
        seed.credits = reader.u32()
        seed.seed = Compartment.load(reader)
        return seed

    def save(self, writer):
        writer.u32(self.credits)
        self.seed.save(writer)

    def __repr__(self):
        return f"Seed {self.credits}"

class Value:
    def __init__(self, meta, name, value):
        self.meta = meta
        self.name = name
        self.value = value

    def save(self, writer):
        self.meta.write(writer, self)

    def __repr__(self):
        return str(f'{self.name}={self.value}')

class Values:
    def __init__(self):
        self.values = []

    def load(self, reader):
        if reader.u8() != 1:
            raise Exception('Unknown version of Values')
        self.values = []
        for i in range( reader.u32() ):
            self.values.append( reader.metastr.read_value(reader) )

    def save(self, writer):
        writer.u8(1)
        writer.array(self.values, Value.save)

    def has_value(self, name):
        for v in self.values:
            if v.name == name:
                return True
        return False

    def get_port(self, name):
        for i,v in enumerate(self.values):
            if v.name == name:
                return i
        return None

class Unit:
    def __init__(self, meta):
        self.meta = meta
        self.ports = []

    def save(self, writer):
        self.meta.write(writer, self)

class Compartment:
    def __init__(self):
        self.compartments = []
        self.units = []
        self.values = Values()

    def load(self, reader):
        if reader.u8() != 1:
            raise Exception('Unknown version of Compartment')
        reader.array(self.compartments, Compartment.load)
        reader.array(self.units, reader.metastr.read_unit)
        self.values.load(reader)

    def save(self, writer):
        writer.u8(1)
        writer.array(self.compartments, Compartment.save)
        writer.array(self.units, Unit.save)
        self.values.save(writer)

    def has_value(self, name):
        return self.values.has_value(name)

class Actor:
    def __init__(self):
        self.id = 0
        self.home = None
        self.credits = 0
        self.reserve = 0
        self.control = Compartment()

    @classmethod
    def load(cls, reader):
        if reader.u8() != 1:
            raise Exception('Unknown version of Actor')
        actor = cls()
        actor.id = reader.u64()
        actor.home = reader.u64()
        actor.credits = reader.u32()
        actor.reserve = reader.u32()
        actor.control.load(reader)
        reader.actors[actor.id] = actor
        return actor

    def update(self, reader):
        self.home = reader.worlds[self.home]

    def save(self, writer):
        writer.u8(1)
        writer.u64(self.id)
        writer.u64(self.home.id)
        writer.u32(self.credits)
        writer.u32(self.reserve)
        self.control.save(writer)

class World:
    def __init__(self):
        self.id = 0
        self.production = 0
        self.actors = []

    @classmethod
    def load(cls, reader):
        world = cls()
        world.id = reader.u64()
        world.production = reader.u32()
        acount = reader.u32()
        for i in range(acount):
            world.actors.append( reader.actors[reader.u64()] )
        reader.worlds[world.id] = world
        return world

    def save(self, writer):
        writer.u64(self.id)
        writer.u32(self.production)
        writer.u32(len(self.actors))
        for a in self.actors:
            writer.u64(a.id)

class Universe:
    def __init__(self):
        self.units = units.load( os.path.dirname(__file__) + '/units.yaml')
        self.timetick = 0
        self.lastseqid = 0
        self.billing = 0
        self.actors = []
        self.worlds = []

    def genid(self):
        self.lastseqid += 1
        return self.lastseqid

    def addworld(self):
        world = World()
        world.id = self.genid()
        self.worlds.append(world)
        return world

    def addactor(self, home):
        actor = Actor()
        actor.home = home
        actor.id = self.genid()
        self.actors.append(actor)
        return actor

    @classmethod
    def load(cls, fs):
        uni = cls()
        reader = Reader(fs, uni.units)
        version = reader.u16()
        if version != UNIVERSE_VERSION:
            raise Exception(f"Unknown version {version}")
        uni.timetick = reader.u64()
        uni.lastseqid = reader.u64()
        uni.billing = reader.u32()
        reader.array(uni.actors, Actor.load)
        reader.array(uni.worlds, World.load)
        for a in uni.actors:
            a.update(reader)
        return uni

    def save(self, fs):
        writer = Writer(fs)
        writer.u16(UNIVERSE_VERSION)
        writer.u64(self.timetick)
        writer.u64(self.lastseqid)
        writer.u32(self.billing)
        writer.array(self.actors, Actor.save)
        writer.array(self.worlds, World.save)

def load(fs):
    return Universe.load(fs)

def load_history(fs):
    return arhistory.History.load(fs)
