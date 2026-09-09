import yaml
import arcadia

class SimpleType:
    def __init__(self, name):
        self.name = name

    def read(self, reader):
        return getattr(reader, self.name)()

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

    def read(self, reader):
        if self.version != reader.u8():
            raise Exception(f'Wrong version of unit {self.id}')
        unit = arcadia.Unit(self)
        for m in self.members:
            setattr(unit, m.name, m.kind.read(reader) )
        return unit

class Structure:
    def __init__(self):
        self.kinds = { 'u32':SimpleType('u32'), 'f32':SimpleType('f32') }
        self.units = {}

    def get_kind(self, typename):
        return self.kinds[typename]

    def load(self, junits):
        for junit in junits:
            unit = Unit.load(junit, self)
            self.units[unit.id] = unit

    def read_unit(self, reader):
        return self.units[reader.u16()].read(reader)

def load(filename):
    with open(filename) as strfile:
        structure = Structure()
        ystr = yaml.load(strfile, Loader=yaml.Loader)
        structure.load(ystr['units'])
        return structure
