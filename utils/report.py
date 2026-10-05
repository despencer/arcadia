#!/usr/bin/env python

import arcadia

def values(actor):
    return "values: "+ ', '.join(map(str, actor.control.values.values))

def unit_params(unit):
    return f'${unit.meta.id}:['+ ', '.join(map(lambda x: str(getattr(unit, x.name))  , unit.meta.members))  +']'

def params(actor):
    return "params: "+ ', '.join(map(unit_params, actor.control.units))

def report(uni):
    print(f'At tick {uni.timetick}, billing {uni.billing}, {len(uni.actors)} actors')
    for w in uni.worlds:
        print(f'World #{w.id}, production {w.production}')
    for a in uni.actors:
        rep = f'Actor #{a.id}, credits {a.credits}/{a.reserve} at #{a.home.id}\n  {values(a)}\n  {params(a)}'
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
