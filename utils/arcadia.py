import struct

msb='little'
UNIVERSE_VERSION=1

CREDIT_SENSOR = 1;
BIRTH_SIGNAL = 2;
BIRTH_CREDIT = 3;
SPAWNER = 4;

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

class BluePrint:
    EMPTY = 0
    FVALUE = 1
    UVALUE = 2
    COLLECTION = 3

    def __init__(self):
        self.value = None

    @classmethod
    def load(cls, reader):
        bp = cls()
        btype = reader.u8()
        if btype == cls.EMPTY:
            pass
        elif btype == cls.FVALUE:
            bp.value = reader.f32()
        elif btype == cls.UVALUE:
            bp.value = reader.u32()
        elif btype == cls.COLLECTION:
            bp.value = []
            for i in range(reader.u32()):
                bp.value.append( cls.load(reader) )
        else:
            raise Exception(f'Unknown blueprint type {btype}')
        return bp

    def save(self, writer):
        if self.value == None:
            writer.u8(cls.EMPTY)
        elif isinstance(self.value, float):
            writer.u8(cls.FVALUE)
            writer.f32(self.value)
        elif isinstance(self.value, int):
            writer.u8(cls.UVALUE)
            writer.u32(self.value)
        elif isinstance(self.value, list):
            writer.u8(cls.COLLECTION)
            writer.u32( len(self.value) )
            for i in self.value:
                i.save(writer)
        else:
            raise Exception(f'Unknown blueprint type {self.value}')

class Seed:
    def __init__(self):
        self.credits = 0
        self.blueprints = BluePrint()

    @classmethod
    def load(cls, reader):
        seed = cls()
        seed.credits = reader.u32()
        seed.blueprints = BluePrint.load(reader)
        return seed

    def save(self, writer):
        writer.u32(self.credits)
        self.blueprints.save(writer)

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

class CreditSensor:
    def __init__(self):
        self.precision = 0

    def load(self, reader):
        self.precision = reader.u32()

    def save(self, writer):
        writer.u16(CREDIT_SENSOR)
        writer.u32(self.precision)

class BirthSignal:
    def __init__(self):
        self.scale = 0
        self.threshold = 0
        self.variation = 0

    def load(self, reader):
        self.scale = reader.f32()
        self.threshold = reader.f32()
        self.variation = reader.u32()

    def save(self, writer):
        writer.u16(BIRTH_SIGNAL)
        writer.f32(self.scale)
        writer.f32(self.threshold)
        writer.u32(self.variation)

class BirthCredit:
    def __init__(self):
        self.giveaway = 0

    def load(self, reader):
        self.giveaway = reader.u32()

    def save(self, writer):
        writer.u16(BIRTH_CREDIT)
        writer.u32(self.giveaway)

class Spawner:
    def __init__(self):
        pass

    def load(self, reader):
        pass

    def save(self, writer):
        writer.u16(SPAWNER)

class Control:
    def __init__(self):
        self.creditsensor = CreditSensor()
        self.birthsignal = BirthSignal()
        self.birthcredit = BirthCredit()
        self.spawner = Spawner()
        self.units = [ self.creditsensor, self.birthsignal, self.birthcredit, self.spawner ]
        self.values = Values()
        self.threshold = 0
        self.giveaway = 0

    def load(self, reader):
        self.units = []
        for i in range( reader.u32() ):
            utype = reader.u16()
            unit = { CREDIT_SENSOR: CreditSensor, BIRTH_SIGNAL: BirthSignal, BIRTH_CREDIT: BirthCredit, SPAWNER: Spawner }[utype]()
            self.units.append(unit)
            if utype == CREDIT_SENSOR:
                self.creditsensor = unit
            elif utype == BIRTH_SIGNAL:
                self.birthsignal = unit
            elif utype == BIRTH_CREDIT:
                self.birthcredit = unit
            elif utype == SPAWNER:
                self.spawner = unit
            unit.load(reader)
        self.values.load(reader)
        self.threshold = reader.u32()
        self.giveaway = reader.u32()

    def save(self, writer):
        writer.u32( len(self.units) )
        for u in self.units:
            u.save(writer)
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
