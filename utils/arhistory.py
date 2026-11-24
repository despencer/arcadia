import os
import storage
import units

HISTORY_VERSION = 1

class Death:
    def __init__(self):
        actorid = 0

    def __repr__(self):
        return f'{self.actorid:08}: Death'

    @classmethod
    def load(cls, reader, version, size):
        if version != 1 or size != 8:
            raise Exception("Bad record")
        death = cls()
        death.actorid = reader.u64()
        return death

class Birth:
    def __init__(self):
        actorid = 0
        parentid = 0
        homeid = 0

    def __repr__(self):
        return f'{self.actorid:08}: Birth from {self.parentid:08} at {self.homeid:04}'

    @classmethod
    def load(cls, reader, version, size):
        if version != 1 or size != 24:
            raise Exception("Bad record")
        birth = cls()
        birth.actorid = reader.u64()
        birth.parentid = reader.u64()
        birth.homeid = reader.u64()
        return birth

class HistoryMessage:
    def __init__(self):
        self.channel = 0
        self.timetick = 0
        self.payload = None

    @classmethod
    def load(cls, reader):
        msg = cls()
        msg.channel = reader.u16()
        msg.timetick = reader.u64()
        msg.payload = reader.metastr.messages['Universe'].read(reader)
        return msg

class History:
    def __init__(self):
        self.messages = []

    @classmethod
    def load(cls, fs):
        fs.seek(0, os.SEEK_END)
        size = fs.tell()
        fs.seek(0)
        reader = storage.Reader(fs, units.load( os.path.dirname(__file__) + '/units.yaml'))
        version = reader.u16()
        if version != HISTORY_VERSION:
            raise Exception(f"Unknown version {version}")
        hist = cls()
        while fs.tell() < size:
            hist.messages.append( HistoryMessage.load(reader) )
        return hist
