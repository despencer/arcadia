#!/usr/bin/env python

import arcadia

def values(actor):
    return "values: " + ', '.join(map(str, actor.control.values.values))

def unit_params(unit):
    return f'${unit.meta.id}:['+ ', '.join(map(lambda x: str(getattr(unit, x.name))  , unit.meta.members))  +']'

def params(actor):
    return "params: "+ ', '.join(map(unit_params, actor.control.units))

def messages(alist):
    return "[" + ', '.join(map(str, alist)) + "]"

rectype = {1: "UNI", 2:"CMN", 3:"RLM", 4:"WRD", 5:"ACT"}

def report(uni):
    print(f'Uni #{uni.id} at tick {uni.timetick}, billing {uni.commune.billing}, {len(uni.commune.actors)} actors, commune #{uni.commune.id}, realm #{uni.realm.id}')
    print(f'Communication Hub, lastid:{uni.commhub.lastseqid} :')
    for r in uni.commhub.receivers.values():
        print(f'   #{r.receiver.id} {rectype[r.receiver.kind]} {messages(r.messages)}')
    for w in uni.realm.worlds:
        print(f'World #{w.id}, production {w.production}')
    for a in uni.commune.actors:
        rep = f'Actor #{a.id}, credits {a.credits}/{a.reserve} at #{a.home}\n  {values(a)}\n  {params(a)}\n  inside:{messages(a.inside)}'
        print(rep)

def main():
    import argparse

    parser = argparse.ArgumentParser(description="Arcadia Universe status report")
    parser.add_argument("filename", type=str, help="File to load")
    args = parser.parse_args()

    with open(args.filename, 'rb') as f:
        uni = arcadia.load(f)
        report(uni)

if __name__ == "__main__":
    main()
