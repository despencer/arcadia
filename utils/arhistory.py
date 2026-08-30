import os
import arcadia

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

class Message:
    def __init__(self):
        self.channel = 0
        self.timetick = 0
        self.payload = None

    @classmethod
    def load(cls, reader):
        msg = cls()
        msg.channel = reader.u16()
        msg.timetick = reader.u64()
        msgclass = reader.u16()
        version = reader.u8()
        msgsize = reader.u16()
        msg.payload = { 1: Death.load, 2: Birth.load }[msgclass](reader, version, msgsize)
        return msg

class History:
    def __init__(self):
        self.messages = []

    @classmethod
    def load(cls, fs):
        fs.seek(0, os.SEEK_END)
        size = fs.tell()
        fs.seek(0)
        reader = arcadia.Reader(fs)
        version = reader.u16()
        if version != HISTORY_VERSION:
            raise Exception(f"Unknown version {version}")
        hist = cls()
        while fs.tell() < size:
            hist.messages.append( Message.load(reader) )
        return hist
