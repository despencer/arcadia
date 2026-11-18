import storage

class Message:
    def __init__(self, meta):
        self.meta = meta

    def save(self, writer):
        self.meta.write(self, writer)

class Receiver:
    UNIVERSE = 1,
    COMMUNE = 2,
    WORLD = 3,
    ACTOR = 4,

    def __init__(self):
        self.kind = 0
        self.id = 0

    @classmethod
    def load(cls, reader):
        party = cls()
        party.kind = reader.u8()
        party.id = reader.u64()
        return party

    def save(self, writer):
        writer.u8(self.kind)
        writer.u64(self.kind)

class Mailbox:
    def __init__(self):
        self.receiver = Receiver()
        self.messages = []

    @classmethod
    def make(cls, id, kind):
        maibox = cls()
        mailbox.receiver.id = id
        mailbox.receiver.kind = kind
        return mailbox

    @classmethod
    def load(cls, reader):
        mailbox = cls()
        mailbox.receiver = Receiver.load(reader)
        reader.array(mailbox.messages, reader.metastr.messages['Universe'].read)
        return mailbox

    def save(self, writer):
        self.receiver.write(writer)
        writer.array(self.messages, Message.save)

class Hub:
    def __init__(self):
        self.lastseqid = 0
        self.receivers = {}

    def genid(self):
        self.lastseqid += 1
        return self.lastseqid

    def receiver(self, kind):
        id = self.genid()
        self.receivers[id] = Mailbox.make(id, kind)
        return id

    @classmethod
    def load(cls, reader):
        hub = cls()
        hub.lastseqid = reader.u64()
        reader.lookup(hub.receivers, reader.u64, Mailbox.load)
        return hub

    def save(self, writer):
        writer.u64(self.lastseqid)
        writer.lookup(self.receiver, writer.u64, Mailbox.save)
