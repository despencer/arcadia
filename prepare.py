#!/usr/bin/env python

msb='little'
UNIVERSE_VERSION=1
UNIVERSE_SEQID = 2
ACTOR_ID = 1
WORLD_ID = 2

def save_actor(fs):
    fs.write( ACTOR_ID.to_bytes(8, msb) ) # Actor ID
    credits = 100
    fs.write( credits.to_bytes(4, msb) )   # amount of credits
    threshold = 500
    fs.write( credits.to_bytes(4, msb) )   # threshold to make a child

def save_world(fs):
    fs.write( WORLD_ID.to_bytes(8, msb) ) # WORLD ID
    production = 1000
    fs.write( production.to_bytes(4, msb) )   # amount of credits per tick
    ucount = 1
    fs.write( ucount.to_bytes(4, msb) )   # number of actors in the world
    fs.write( ACTOR_ID.to_bytes(8, msb) ) # Actor ID

def save(fs):
    fs.write( UNIVERSE_VERSION.to_bytes(2, msb) )
    ticks = 0
    fs.write( ticks.to_bytes(8, msb) )
    fs.write( UNIVERSE_SEQID.to_bytes(8, msb) )
    billing = 10
    fs.write( billing.to_bytes(4, msb) )
    ucount = 1
    fs.write( ucount.to_bytes(4, msb) )   # number of actors
    save_actor(fs)
    fs.write( ucount.to_bytes(4, msb) )   # number of worlds
    save_world(fs)

def main():
    import argparse

    parser = argparse.ArgumentParser(description="Produces Arcadia Universe")
    parser.add_argument("filename", type=str, help="File to create")
    args = parser.parse_args()

    with open(args.filename, 'wb') as f:
        save(f)

if __name__ == "__main__":
    main()
