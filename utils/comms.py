import storage

class Message:
    def __init__(self, meta):
        self.meta = meta

    def save(self, writer):
        self.meta.write(self, writer)

class Party:
    WORLD = 1

    def __init__(self):
        self.kind = 0
        self.id = 0

    @classmethod
    def load(cls, reader):
        party = cls()
        party.kind = reader.u8()
        if party.kind == self.WORLD:
            party.id = reader.u64()
        else:
            raise Exception(f'Unknown Party type {party.kind}')
        return party

    def save(self, writer):
        writer.u8(self.kind)
        if self.kind == self.WORLD:
            writer.u64(self.kind)
        else:
            raise Exception(f'Unknown Party type {self.kind}')

class Channel:
    def __init__(self):
        self.sender = Party()
        self.receiver = Party()
        self.messages = []

    @classmethod
    def load(cls, reader):
        channel = cls()
        channel.sender = Party.load(reader)
        channel.receiver = Party.load(reader)
        reader.array(channel.messages, reader.metastr.messages['Universe'].read)
        return channel

    def save(self, writer):
        self.sender.write(writer)
        self.receiver.write(writer)
        writer.array(self.messages, Message.save)

class Hub:
    def __init__(self):
        self.channels = storage.Depot()

    @classmethod
    def load(cls, reader):
        hub = cls()
        hub.channels = storage.Depot.load(reader, Channel.load)
        return hub

    def save(self, writer):
        self.channels.save(writer, Channel.save)
