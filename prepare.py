#!/usr/bin/env python

msb='little'
UNIVERSE_VERSION=1
UNIVERSE_SEQID = 1

def save_actor(fs):
    fs.write( UNIVERSE_SEQID.to_bytes(8, msb) )

def save(fs):
    fs.write( UNIVERSE_VERSION.to_bytes(2, msb) )
    ticks = 0
    fs.write( ticks.to_bytes(8, msb) )
    fs.write( UNIVERSE_SEQID.to_bytes(8, msb) )
    ucount = 1
    fs.write( ucount.to_bytes(4, msb) )
    save_actor(fs)

def main():
    import argparse

    parser = argparse.ArgumentParser(description="Produces Arcadia Universe")
    parser.add_argument("filename", type=str, help="File to create")
    args = parser.parse_args()

    with open(args.filename, 'wb') as f:
        save(f)

if __name__ == "__main__":
    main()
