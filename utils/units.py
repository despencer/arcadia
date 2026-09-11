import yaml
import arcadia

class SimpleType:
    def __init__(self, name):
        self.name = name

    def read(self, reader):
        return getattr(reader, self.name)()

    def write(self, writer, value):
        getattr(writer, self.name)(value)

class Member:
    def __init__(self):
        self.kind = None
        self.name = ''

    @classmethod
    def load(cls, jm, structure):
        m = cls()
        m.kind = structure.get_kind(jm['type'])
        m.name = jm['name']
        return m

class Unit:
    def __init__(self):
        self.id = 0
        self.version = 1
        self.name = ''
        self.members = []

    @classmethod
    def load(cls, junit, structure):
        unit = cls()
        unit.id = junit['id']
        unit.name = junit['name']
        for jm in junit['data']:
            unit.members.append( Member.load(jm, structure) )
        return unit

    def make(self, junitdef):
        unit = arcadia.Unit(self)
        for m in self.members:
            for jm in junitdef['data']:
                if jm['name'] == m.name:
                    setattr(unit, m.name, jm['value'])
        return unit

    def read(self, reader):
        if self.version != reader.u8():
            raise Exception(f'Wrong version of unit {self.id}')
        unit = arcadia.Unit(self)
        for m in self.members:
            setattr(unit, m.name, m.kind.read(reader) )
        return unit

    def write(self, writer, unit):
        writer.u16(self.id)
        writer.u8(self.version)
        for m in self.members:
            m.kind.write(writer, getattr(unit, m.name))

class Structure:
    def __init__(self):
        self.kinds = { 'u32':SimpleType('u32'), 'f32':SimpleType('f32') }
        self.unitids = {}
        self.unitnames = {}

    def get_kind(self, typename):
        return self.kinds[typename]

    def load(self, junits):
        for junit in junits:
            unit = Unit.load(junit, self)
            self.unitids[unit.id] = unit
            self.unitnames[unit.name] = unit

    def make_unit(self, junitdef):
        return self.unitnames[ junitdef['type'] ].make(junitdef)

    def read_unit(self, reader):
        return self.unitids[reader.u16()].read(reader)

def load(filename):
    with open(filename) as strfile:
        structure = Structure()
        ystr = yaml.load(strfile, Loader=yaml.Loader)
        structure.load(ystr['units'])
        return structure
