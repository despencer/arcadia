import yaml
import arcadia

class SimpleType:
    def __init__(self, name, default):
        self.name = name
        self.default = default

    def make(self, jvalue):
        return jvalue

    def read(self, reader):
        return getattr(reader, self.name)()

    def write(self, writer, value):
        getattr(writer, self.name)(value)

class CommonType:
    def __init__(self, name, metastr, aclass):
        self.name = name
        self.aclass = aclass
        self.metastr = metastr

    def make(self, jvalue):
        return self.aclass.make_from(self.metastr, jvalue)

    def read(self, reader):
        return self.aclass.load(self.metastr, reader)

    def write(self, writer, value):
        return value.save(writer)

class ArrayType:
    def __init__(self, name, itemtype):
        self.name = name
        self.default = []
        self.itemtype = itemtype

    def read(self, reader):
        result = []
        reader.array(result, self.itemtype.read)
        return result

    def write(self, writer, value):
        writer.array(value, self.itemtype.read)

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

class Port:
    def __init__(self):
        self.name = ''
        self.value = None

    @classmethod
    def load(cls, jp, structure):
        p = cls()
        p.name = jp['name']
        p.value = structure.get_value(jp['type'])
        return p

class Unit:
    def __init__(self):
        self.id = 0
        self.version = 1
        self.name = ''
        self.members = []
        self.ports = []

    @classmethod
    def load(cls, junit, structure):
        unit = cls()
        unit.id = junit['id']
        unit.name = junit['name']
        for jm in junit['data']:
            unit.members.append( Member.load(jm, structure) )
        if 'ports' in junit:
            for jp in junit['ports']:
                unit.ports.append( Port.load(jp, structure) )
        return unit

    def make(self, junitdef):
        unit = arcadia.Unit(self)
        for m in self.members:
            for jm in junitdef['data']:
                if jm['name'] == m.name:
                    setattr(unit, m.name, m.kind.make(jm['value']))
        return unit

    def read(self, reader):
        if self.version != reader.u8():
            raise Exception(f'Wrong version of unit {self.id}')
        unit = arcadia.Unit(self)
        for m in self.members:
            setattr(unit, m.name, m.kind.read(reader) )
        for p in self.ports:
            unit.ports.append( reader.u8() )
        return unit

    def write(self, writer, unit):
        writer.u16(self.id)
        writer.u8(self.version)
        for m in self.members:
            m.kind.write(writer, getattr(unit, m.name))
        for p in unit.ports:
            writer.u8(p)

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

    def make(self, name):
        return arcadia.Value(self, name, self.kind.default)

    def read(self, reader, name):
        return arcadia.Value(self, name, self.kind.read(reader))

    def write(self, writer, value):
        writer.utf8(value.name)
        writer.u8(self.id)
        self.kind.write(writer, value.value)

class Structure:
    def __init__(self):
        self.kinds = { 'u32':SimpleType('u32', 0), 'f32':SimpleType('f32', 0.0), 'bool':SimpleType('bool', False),
                       'seeds': ArrayType('seeds', CommonType('Seed', self, arcadia.Seed)), 'blueprint': CommonType('Blueprint', self, arcadia.BluePrint) }
        self.unitids = {}
        self.unitnames = {}
        self.valueids = {}
        self.valuetypes = {}

    def get_kind(self, typename):
        return self.kinds[typename]

    def get_unit(self, unitname):
        return self.unitnames[unitname]

    def get_value(self, typename):
        return self.valuetypes[typename]

    def load(self, jstr):
        for jvalue in jstr['values']:
            value = Value.load(jvalue, self)
            self.valueids[value.id] = value
            self.valuetypes[value.kind.name] = value
        for junit in jstr['units']:
            unit = Unit.load(junit, self)
            self.unitids[unit.id] = unit
            self.unitnames[unit.name] = unit

    def make_unit(self, junitdef, control):
        unit = self.unitnames[ junitdef['type'] ].make(junitdef)
        control.units.append(unit)
        for port in self.unitnames[ junitdef['type'] ].ports:
            if not control.has_value(port.name):
                control.values.values.append(port.value.make(port.name))
            unit.ports.append(control.values.get_port(port.name))

    def read_unit(self, reader):
        return self.unitids[reader.u16()].read(reader)

    def read_value(self, reader):
        name = reader.utf8()
        return self.valueids[reader.u8()].read(reader, name)

def load(filename):
    with open(filename) as strfile:
        structure = Structure()
        ystr = yaml.load(strfile, Loader=yaml.Loader)
        structure.load(ystr)
        return structure
