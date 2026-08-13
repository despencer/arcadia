import struct

msb='little'
UNIVERSE_VERSION=1

class Reader:
    def __init__(self, fs):
        self.fs = fs
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

    def bl(self):
        return (self.u8() != 0)

    def array(self, alist, reader):
        acount = self.u32()
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

    def bl(self, value):
        if value:
            self.u8(1)
        else:
            self.u8(0)

    def array(self, alist, writer):
        self.u32(len(alist))
        for item in alist:
            writer(item, self)

class Seed:
    def __init__(self):
        self.credits = 0

    @classmethod
    def load(cls, reader):
        seed = cls()
        seed.credits = reader.u32()
        return seed

    def save(self, writer):
        writer.u32(self.credits)

    def __repr__(self):
        return f"Seed {self.credits}"

class Values:
    def __init__(self):
        self.credits = 0.0
        self.birth = False
        self.birthcredits = []

    def load(self, reader):
        self.credits = reader.f32()
        self.birth = reader.bl()
        reader.array(self.birthcredits, Seed.load)

    def save(self, writer):
        writer.f32(self.credits)
        writer.bl(self.birth)
        writer.array(self.birthcredits, Seed.save)

class BirthSignal:
    def __init__(self):
        self.scale = 0
        self.threshold = 0
        self.variation = 0

    def load(self, reader):
        self.scale = reader.f32()
        self.threshold = reader.f32()
        self.variation = reader.f32()

    def save(self, writer):
        writer.f32(self.scale)
        writer.f32(self.threshold)
        writer.f32(self.variation)

class Control:
    def __init__(self):
        self.creditsensor = 0
        self.birthsignal = BirthSignal()
        self.birthgiveaway = 0
        self.values = Values()
        self.threshold = 0
        self.giveaway = 0

    def load(self, reader):
        self.creditsensor = reader.u32()
        self.birthsignal.load(reader)
        self.birthgiveaway = reader.u32()
        self.values.load(reader)
        self.threshold = reader.u32()
        self.giveaway = reader.u32()

    def save(self, writer):
        writer.u32(self.creditsensor)
        self.birthsignal.save(writer)
        writer.u32(self.birthgiveaway)
        self.values.save(writer)
        writer.u32(self.threshold)
        writer.u32(self.giveaway)

class Actor:
    def __init__(self):
        self.id = 0
        self.home = None
        self.credits = 0
        self.reserve = 0
        self.control = Control()

    @classmethod
    def load(cls, reader):
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
        reader = Reader(fs)
        version = reader.u16()
        if version != UNIVERSE_VERSION:
            raise Exception(f"Unknown version {version}")
        uni = cls()
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
