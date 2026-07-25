msb='little'
UNIVERSE_VERSION=1

class Reader:
    def __init__(self, fs):
        self.fs = fs
        self.actors = {}
        self.worlds = {}

    def read_u(self, size):
        return int.from_bytes(self.fs.read(size), msb)

    def u16(self):
        return self.read_u(2)

    def u32(self):
        return self.read_u(4)

    def u64(self):
        return self.read_u(8)

    def array(self, alist, reader):
        acount = self.u32()
        for i in range(acount):
            alist.append( reader(self) )

class Control:
    def __init__(self):
        self.threshold = 0
        self.giveaway = 0

class Actor:
    def __init__(self):
        self.id = 0
        self.home = None
        self.credits = 0
        self.control = Control()

    @classmethod
    def load(cls, reader):
        actor = cls()
        actor.id = reader.u64()
        actor.home = reader.u64()
        actor.credits = reader.u32()
        actor.control.threshold = reader.u32()
        actor.control.giveaway = reader.u32()
        reader.actors[actor.id] = actor
        return actor

    def update(self, reader):
        self.home = reader.worlds[self.home]

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

class Universe:
    def __init__(self):
        self.timetick = 0
        self.lastseqid = 1
        self.billing = 0
        self.actors = []
        self.worlds = []

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

def load(fs):
    return Universe.load(fs)
