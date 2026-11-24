import storage

class Message:
    def __init__(self, meta):
        self.meta = meta

    def save(self, writer):
        self.meta.write(self, writer)

    def __repr__(self):
        return self.meta.present(self)

class Receiver:
    UNIVERSE = 1
    COMMUNE = 2
    REALM = 3
    WORLD = 4
    ACTOR = 5

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
        writer.u64(self.id)

class Mailbox:
    def __init__(self):
        self.receiver = Receiver()
        self.messages = []

    @classmethod
    def make(cls, mid, kind):
        mailbox = cls()
        mailbox.receiver.id = mid
        mailbox.receiver.kind = kind
        return mailbox

    @classmethod
    def load(cls, reader):
        mailbox = cls()
        mailbox.receiver = Receiver.load(reader)
        reader.array(mailbox.messages, reader.metastr.messages['Universe'].read)
        return mailbox

    def save(self, writer):
        self.receiver.save(writer)
        writer.array(self.messages, Message.save)

class Hub:
    def __init__(self):
        self.lastseqid = 0
        self.receivers = {}

    def genid(self):
        self.lastseqid += 1
        return self.lastseqid

    def genreceiver(self, kind):
        mid = self.genid()
        self.receivers[mid] = Mailbox.make(mid, kind)
        return mid

    @classmethod
    def load(cls, reader):
        hub = cls()
        hub.lastseqid = reader.u64()
        reader.lookup(hub.receivers, storage.read_id, Mailbox.load)
        return hub

    def save(self, writer):
        writer.u64(self.lastseqid)
        writer.lookup(self.receivers, storage.write_id, Mailbox.save)
