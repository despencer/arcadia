#!/usr/bin/env python

import arcadia

def unit_params(unit):
    return '['+ ' ,'.join(map(lambda x: str(getattr(unit, x.name))  , unit.meta.members))  +']'

def params(actor, units):
    return "["+ ' ,'.join(map(unit_params, actor.control.units)) +"]"

def report(uni):
    print(f'At tick {uni.timetick}, billing {uni.billing}, {len(uni.actors)} actors')
    for w in uni.worlds:
        print(f'World #{w.id}, production {w.production}')
    for a in uni.actors:
        rep = f'Actor #{a.id}, credits {a.credits}/{a.reserve} (cr={a.control.values.credits}, birth={a.control.values.birth}, bcred={a.control.values.birthcredits}, seeds={a.control.values.seeds}) at #{a.home.id}'
        rep += f' params {params(a, uni.units.units)}'
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
