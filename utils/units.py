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

class Value:
    def __init__(self):
        self.id = 0
        self.kind = None

    @classmethod
    def load(cls, jv, structure):
        v = cls()
        v.id = jv['id']
        v.kind = structure.get_kind(jv['type'])
        return v

    def make(self, jvaluedef):
        return arcadia.Value(self, jvaluedef['name'], jvaluedef['value'])

    def read(self, reader, name):
        return arcadia.Value(self, name, self.kind.read(reader))

    def write(self, writer, value):
        writer.utf8(value.name)
        writer.u8(self.id)
        self.kind.write(writer, value.value)

class Structure:
    def __init__(self):
        self.kinds = { 'u32':SimpleType('u32'), 'f32':SimpleType('f32') }
        self.unitids = {}
        self.unitnames = {}
        self.valueids = {}
        self.valuetypes = {}

    def get_kind(self, typename):
        return self.kinds[typename]

    def load(self, jstr):
        for junit in jstr['units']:
            unit = Unit.load(junit, self)
            self.unitids[unit.id] = unit
            self.unitnames[unit.name] = unit
        for jvalue in jstr['values']:
            value = Value.load(jvalue, self)
            self.valueids[value.id] = value
            self.valuetypes[value.kind.name] = value

    def make_unit(self, junitdef):
        return self.unitnames[ junitdef['type'] ].make(junitdef)

    def read_unit(self, reader):
        return self.unitids[reader.u16()].read(reader)

    def make_value(self, jvaluedef):
        return self.valuetypes[ jvaluedef['type'] ].make(jvaluedef)

    def read_value(self, reader):
        name = reader.utf8()
        return self.valueids[reader.u8()].read(reader, name)

def load(filename):
    with open(filename) as strfile:
        structure = Structure()
        ystr = yaml.load(strfile, Loader=yaml.Loader)
        structure.load(ystr)
        return structure
