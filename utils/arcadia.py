import os
import arhistory
import units
import comms
import storage

UNIVERSE_VERSION=1

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
        seed.seed.load(reader)
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
        self.meta.write(self, writer)

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
        self.commune = 0
        self.home = 0
        self.credits = 0
        self.reserve = 0
        self.inside = []
        self.control = Compartment()

    @classmethod
    def load(cls, reader):
        if reader.u8() != 1:
            raise Exception('Unknown version of Actor')
        actor = cls()
        actor.id = reader.u64()
        actor.commune = reader.u64()
        actor.home = reader.u64()
        actor.credits = reader.u32()
        actor.reserve = reader.u32()
        reader.array(actor.inside, reader.metastr.messages['ActorInside'].read)
        actor.control.load(reader)
        return actor

    def save(self, writer):
        writer.u8(1)
        writer.u64(self.id)
        writer.u64(self.commune)
        writer.u64(self.home)
        writer.u32(self.credits)
        writer.u32(self.reserve)
        writer.array(self.inside, comms.Message.save)
        self.control.save(writer)

class World:
    def __init__(self):
        self.id = 0
        self.production = 0
        self.actors = []

    def addactor(self, aid):
        self.actors.append(aid)

    @classmethod
    def load(cls, reader):
        world = cls()
        world.id = reader.u64()
        world.production = reader.u32()
        reader.array(world.actors, storage.read_id)
        return world

    def save(self, writer):
        writer.u64(self.id)
        writer.u32(self.production)
        writer.array(self.actors, storage.write_id)

class Commune:
    def __init__(self):
        self.id = 0
        self.billing = 0
        self.actors = []

    def addactor(self, homeid, commhub):
        actor = Actor()
        actor.id = commhub.genreceiver(comms.Receiver.ACTOR)
        actor.home = homeid
        actor.commune = self.id
        self.actors.append(actor)
        return actor

    @classmethod
    def load(cls, reader):
        cmn = cls()
        cmn.id = reader.u64()
        cmn.billing = reader.u32()
        reader.array(cmn.actors, Actor.load)
        return cmn

    def save(self, writer):
        writer.u64(self.id)
        writer.u32(self.billing)
        writer.array(self.actors, Actor.save)

class Realm:
    def __init__(self):
        self.id = 0
        self.worlds = []

    def addworld(self, commhub):
        world = World()
        world.id = commhub.genreceiver(comms.Receiver.WORLD)
        self.worlds.append(world)
        return world

    @classmethod
    def load(cls, reader):
        realm = cls()
        realm.id = reader.u64()
        reader.array(realm.worlds, World.load)
        return realm

    def save(self, writer):
        writer.u64(self.id)
        writer.array(self.worlds, World.save)

class Universe:
    def __init__(self):
        self.units = units.load( os.path.dirname(__file__) + '/units.yaml')
        self.timetick = 0
        self.commune = Commune()
        self.realm = Realm()
        self.commhub = comms.Hub()

    def addworld(self):
        return self.realm.addworld(self.commhub)

    def addactor(self, home):
        actor = self.commune.addactor(home.id, self.commhub)
        home.addactor(actor.id)
        return actor

    @classmethod
    def make(cls):
        uni = cls()
        uni.commhub.lastseqid = 1
        uni.id = uni.commhub.genreceiver(comms.Receiver.UNIVERSE)
        uni.commune.id = uni.commhub.genreceiver(comms.Receiver.COMMUNE)
        uni.realm.id = uni.commhub.genreceiver(comms.Receiver.REALM)
        return uni

    @classmethod
    def load(cls, fs):
        uni = cls()
        reader = storage.Reader(fs, uni.units)
        version = reader.u16()
        if version != UNIVERSE_VERSION:
            raise Exception(f"Unknown version {version}")
        uni.id = reader.u64()
        uni.timetick = reader.u64()
        uni.commune = Commune.load(reader)
        uni.realm = Realm.load(reader)
        uni.commhub = comms.Hub.load(reader)
        return uni

    def save(self, fs):
        writer = storage.Writer(fs)
        writer.u16(UNIVERSE_VERSION)
        writer.u64(self.id)
        writer.u64(self.timetick)
        self.commune.save(writer)
        self.realm.save(writer)
        self.commhub.save(writer)

def load(fs):
    return Universe.load(fs)

def load_history(fs):
    return arhistory.History.load(fs)
